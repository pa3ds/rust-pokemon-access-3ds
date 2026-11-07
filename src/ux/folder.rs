//! The folder picker: pa3ds asks for a portable Azahar's folder with it, where it found none.
//!
//! Tk's `askdirectory`, which on Windows is the shell's own `IFileOpenDialog` picking folders
//! (`FOS_PICKFOLDERS`): the dialog any program's folder picker is, read by a screen reader as it
//! reads Explorer's, with its title said. Tk draws none of it; only the hidden root it is opened over
//! is Tk's. The port calls the same dialog through COM.

use std::path::{Path, PathBuf};

/// The folder the player picks, starting in `initial`, or None when they cancel or no picker
/// can open, which is said in `notes`.
pub fn choose(title: &str, initial: &Path, notes: &mut Vec<String>) -> Option<PathBuf> {
    match picker::choose(title, initial) {
        Ok(chosen) => chosen,
        Err(error) => {
            notes.push(format!("no folder picker could open: {error}"));
            None
        }
    }
}

/// The shell's `IFileOpenDialog` picking folders, reached through COM. windows-sys declares no COM interfaces, so the two used are
/// declared here, each vtable in `shobjidl_core.h`'s order up to the last slot called.
#[cfg(windows)]
mod picker {
    use std::ffi::{OsString, c_void};
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use std::path::{Path, PathBuf};
    use std::ptr::{null, null_mut};

    use windows_sys::Win32::Foundation::{ERROR_CANCELLED, HWND};
    use windows_sys::Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoCreateInstance,
        CoInitializeEx, CoTaskMemFree, CoUninitialize,
    };
    use windows_sys::Win32::UI::Shell::{
        FOS_FORCEFILESYSTEM, FOS_PATHMUSTEXIST, FOS_PICKFOLDERS, FileOpenDialog,
        SHCreateItemFromParsingName, SIGDN_FILESYSPATH,
    };
    use windows_sys::core::{GUID, HRESULT, PCWSTR, PWSTR};

    const IID_IFILEOPENDIALOG: GUID = GUID::from_u128(0xd57c7288_d4ad_4768_be02_9d969532d960);
    const IID_ISHELLITEM: GUID = GUID::from_u128(0x43826d1e_e718_42ee_bc55_a1e261c37bfe);
    /// `HRESULT_FROM_WIN32(ERROR_CANCELLED)`, what `Show` returns when the player cancels.
    const CANCELLED: HRESULT = (0x8007_0000 | ERROR_CANCELLED) as HRESULT;

    type This = *mut c_void;

    #[repr(C)]
    struct UnknownVtbl {
        _query_interface: usize,
        _add_ref: usize,
        release: unsafe extern "system" fn(This) -> u32,
    }

    #[repr(C)]
    struct FileDialogVtbl {
        _unknown: UnknownVtbl,
        show: unsafe extern "system" fn(This, HWND) -> HRESULT,
        _set_file_types: usize,
        _set_file_type_index: usize,
        _get_file_type_index: usize,
        _advise: usize,
        _unadvise: usize,
        set_options: unsafe extern "system" fn(This, u32) -> HRESULT,
        get_options: unsafe extern "system" fn(This, *mut u32) -> HRESULT,
        _set_default_folder: usize,
        set_folder: unsafe extern "system" fn(This, This) -> HRESULT,
        _get_folder: usize,
        _get_current_selection: usize,
        _set_file_name: usize,
        _get_file_name: usize,
        set_title: unsafe extern "system" fn(This, PCWSTR) -> HRESULT,
        _set_ok_button_label: usize,
        _set_file_name_label: usize,
        get_result: unsafe extern "system" fn(This, *mut This) -> HRESULT,
    }

    #[repr(C)]
    struct ShellItemVtbl {
        _unknown: UnknownVtbl,
        _bind_to_handler: usize,
        _get_parent: usize,
        get_display_name: unsafe extern "system" fn(This, i32, *mut PWSTR) -> HRESULT,
    }

    /// A COM object, released when dropped.
    struct Com(This);

    impl Com {
        /// # Safety
        /// `V` must be the vtable of the interface this pointer was asked for.
        unsafe fn vtbl<V>(&self) -> &V {
            unsafe { &**(self.0 as *const *const V) }
        }
    }

    impl Drop for Com {
        fn drop(&mut self) {
            unsafe { (self.vtbl::<UnknownVtbl>().release)(self.0) };
        }
    }

    /// COM for this thread, let go when dropped where this call is what took it.
    struct Apartment(bool);

    impl Drop for Apartment {
        fn drop(&mut self) {
            if self.0 {
                unsafe { CoUninitialize() };
            }
        }
    }

    fn check(result: HRESULT) -> Result<(), String> {
        if result < 0 {
            return Err(std::io::Error::from_raw_os_error(result).to_string());
        }
        Ok(())
    }

    fn wide(text: &std::ffi::OsStr) -> Vec<u16> {
        text.encode_wide().chain([0]).collect()
    }

    /// The folder the player picked, starting in `initial`; None when they cancel.
    pub fn choose(title: &str, initial: &Path) -> Result<Option<PathBuf>, String> {
        let flags = (COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) as u32;
        let _apartment = Apartment(unsafe { CoInitializeEx(null(), flags) } >= 0);
        let mut made: This = null_mut();
        check(unsafe {
            CoCreateInstance(
                &FileOpenDialog,
                null_mut(),
                CLSCTX_INPROC_SERVER,
                &IID_IFILEOPENDIALOG,
                &mut made,
            )
        })?;
        let dialog = Com(made);
        let calls = unsafe { dialog.vtbl::<FileDialogVtbl>() };
        let mut options = 0;
        check(unsafe { (calls.get_options)(dialog.0, &mut options) })?;
        let options = options | FOS_PICKFOLDERS | FOS_PATHMUSTEXIST | FOS_FORCEFILESYSTEM;
        check(unsafe { (calls.set_options)(dialog.0, options) })?;
        let title = wide(title.as_ref());
        check(unsafe { (calls.set_title)(dialog.0, title.as_ptr()) })?;
        // Tk starts wherever it can when the initial folder will not do, and so does this.
        let mut folder: This = null_mut();
        let path = wide(initial.as_os_str());
        let found = unsafe {
            SHCreateItemFromParsingName(path.as_ptr(), null_mut(), &IID_ISHELLITEM, &mut folder)
        };
        if found >= 0 {
            let folder = Com(folder);
            unsafe { (calls.set_folder)(dialog.0, folder.0) };
        }
        let shown = unsafe { (calls.show)(dialog.0, null_mut()) };
        if shown == CANCELLED {
            return Ok(None);
        }
        check(shown)?;
        let mut picked: This = null_mut();
        check(unsafe { (calls.get_result)(dialog.0, &mut picked) })?;
        let picked = Com(picked);
        let mut name: PWSTR = null_mut();
        check(unsafe {
            (picked.vtbl::<ShellItemVtbl>().get_display_name)(
                picked.0,
                SIGDN_FILESYSPATH,
                &mut name,
            )
        })?;
        let chosen = unsafe {
            let length = (0..).take_while(|&at| *name.add(at) != 0).count();
            let chosen = OsString::from_wide(std::slice::from_raw_parts(name, length));
            CoTaskMemFree(name as *const c_void);
            chosen
        };
        Ok(Some(PathBuf::from(chosen)))
    }
}

/// On macOS and Linux, the system's own folder picker (`dialog::native`).
#[cfg(not(windows))]
mod picker {
    pub use crate::ux::dialog::native::folder as choose;
}
