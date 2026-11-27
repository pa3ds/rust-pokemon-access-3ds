//! A program's slider in Windows' volume mixer, read through the Core Audio sessions.
//!
//! Each program's sound is a session on the output it plays to, and the mixer's slider for it is
//! the session's volume, its mute button the session's mute (`ISimpleAudioVolume`). pa3ds reads
//! Azahar's so a sound of its own is as loud as Azahar's would be (`clips`). Plain COM through
//! ctypes, a method by its place in the interface's table: the interfaces are Windows' own and
//! their tables do not change. Read on the thread that asks, which COM is started on.

/// The mixer's slider for the first session of the program whose file is `program`
/// (`azahar.exe`) on any output, of process `pid` alone where one is given, 0 while muted; None
/// where it has none or COM fails. With two Azahars open, the first was the other one's, and
/// pa3ds's sounds played at its 100% beside a game at 8%.
///
/// Python's `pid` defaults to 0.
pub fn slider(program: &str, pid: i64) -> Option<f64> {
    com::slider(&program.to_lowercase(), pid).ok().flatten()
}

#[cfg(windows)]
mod com {
    use std::ffi::c_void;
    use std::ptr::null_mut;

    use windows_sys::Win32::System::Com::{CLSCTX_ALL, CoCreateInstance, CoInitializeEx};
    use windows_sys::core::{GUID, HRESULT};

    use crate::core::window;

    const ENUMERATOR: GUID = GUID::from_u128(0xBCDE0395_E52F_467C_8E3D_C4579291692E);
    const IMM_DEVICE_ENUMERATOR: GUID = GUID::from_u128(0xA95664D2_9614_4F35_A746_DE8DB63617E6);
    const IAUDIO_SESSION_MANAGER2: GUID = GUID::from_u128(0x77AA99A0_1BD6_484F_8BC7_2C654C9A9B6F);
    const IAUDIO_SESSION_CONTROL2: GUID = GUID::from_u128(0xBFB7FF88_7239_4FC9_8FA2_07C950BE9C6D);
    const ISIMPLE_AUDIO_VOLUME: GUID = GUID::from_u128(0x87CE5498_68D6_44E5_9215_6DA47EF883D8);
    const RENDER: i32 = 0;
    const ACTIVE: u32 = 1;
    // The methods used, by their place in each interface's table (IUnknown's three come first).
    const QUERY: usize = 0;
    const RELEASE: usize = 2;
    const ENUM_ENDPOINTS: usize = 3;
    const COUNT: usize = 3;
    const ITEM: usize = 4;
    const ACTIVATE: usize = 3;
    const SESSION_ENUMERATOR: usize = 5;
    const SESSION_COUNT: usize = 3;
    const SESSION: usize = 4;
    const PROCESS_ID: usize = 14;
    const GET_VOLUME: usize = 4;
    const GET_MUTE: usize = 6;

    type Object = *mut c_void;

    /// A COM call that failed, as ctypes raises OSError for an HRESULT below zero.
    pub struct Failed;

    fn checked(result: HRESULT) -> Result<(), Failed> {
        if result < 0 { Err(Failed) } else { Ok(()) }
    }

    /// One of a COM object's methods by its place in its table, as the type `F` the method has.
    ///
    /// # Safety
    /// `target` is a live COM object whose table has a method of type `F` at `index`.
    unsafe fn method<F: Copy>(target: Object, index: usize) -> F {
        // SAFETY: a COM object starts with a pointer to its table of function pointers.
        unsafe {
            let table = *(target as *const *const usize);
            std::mem::transmute_copy::<usize, F>(&*table.add(index))
        }
    }

    /// Releases what it holds as it goes, as the Python's `finally` does.
    struct Held(Object);

    impl Drop for Held {
        fn drop(&mut self) {
            if !self.0.is_null() {
                // SAFETY: a non-null `Held` is a live COM object, released once.
                unsafe {
                    method::<unsafe extern "system" fn(Object) -> u32>(self.0, RELEASE)(self.0);
                }
            }
        }
    }

    pub fn slider(program: &str, wanted: i64) -> Result<Option<f64>, Failed> {
        // SAFETY: every call below is handed live objects and places to write that outlive it.
        unsafe {
            // Started already on this thread, in another mode, is no failure.
            let _ = CoInitializeEx(std::ptr::null(), 0);
            let mut enumerator = Held(null_mut());
            checked(CoCreateInstance(
                &ENUMERATOR,
                null_mut(),
                CLSCTX_ALL,
                &IMM_DEVICE_ENUMERATOR,
                &mut enumerator.0,
            ))?;
            let mut devices = Held(null_mut());
            checked(method::<
                unsafe extern "system" fn(Object, i32, u32, *mut Object) -> HRESULT,
            >(enumerator.0, ENUM_ENDPOINTS)(
                enumerator.0,
                RENDER,
                ACTIVE,
                &mut devices.0,
            ))?;
            let mut count: u32 = 0;
            checked(method::<
                unsafe extern "system" fn(Object, *mut u32) -> HRESULT,
            >(devices.0, COUNT)(devices.0, &mut count))?;
            for index in 0..count {
                let found = on_device(&devices, index, program, wanted)?;
                if found.is_some() {
                    return Ok(found);
                }
            }
            Ok(None)
        }
    }

    unsafe fn on_device(
        devices: &Held,
        index: u32,
        program: &str,
        wanted: i64,
    ) -> Result<Option<f64>, Failed> {
        // SAFETY: as `slider`'s.
        unsafe {
            let mut device = Held(null_mut());
            checked(method::<
                unsafe extern "system" fn(Object, u32, *mut Object) -> HRESULT,
            >(devices.0, ITEM)(
                devices.0, index, &mut device.0
            ))?;
            let mut manager = Held(null_mut());
            checked(method::<
                unsafe extern "system" fn(
                    Object,
                    *const GUID,
                    u32,
                    *mut c_void,
                    *mut Object,
                ) -> HRESULT,
            >(device.0, ACTIVATE)(
                device.0,
                &IAUDIO_SESSION_MANAGER2,
                CLSCTX_ALL,
                null_mut(),
                &mut manager.0,
            ))?;
            let mut sessions = Held(null_mut());
            checked(method::<
                unsafe extern "system" fn(Object, *mut Object) -> HRESULT,
            >(manager.0, SESSION_ENUMERATOR)(
                manager.0, &mut sessions.0
            ))?;
            let mut count: i32 = 0;
            checked(method::<
                unsafe extern "system" fn(Object, *mut i32) -> HRESULT,
            >(sessions.0, SESSION_COUNT)(
                sessions.0, &mut count
            ))?;
            for at in 0..count {
                let found = session(&sessions, at, program, wanted)?;
                if found.is_some() {
                    return Ok(found);
                }
            }
            Ok(None)
        }
    }

    unsafe fn session(
        sessions: &Held,
        at: i32,
        program: &str,
        wanted: i64,
    ) -> Result<Option<f64>, Failed> {
        // SAFETY: as `slider`'s.
        unsafe {
            let query = |object: Object, iid: &GUID, out: &mut Object| {
                checked(method::<
                    unsafe extern "system" fn(Object, *const GUID, *mut Object) -> HRESULT,
                >(object, QUERY)(object, iid, out))
            };
            let mut control = Held(null_mut());
            checked(method::<
                unsafe extern "system" fn(Object, i32, *mut Object) -> HRESULT,
            >(sessions.0, SESSION)(
                sessions.0, at, &mut control.0
            ))?;
            let mut named = Held(null_mut());
            query(control.0, &IAUDIO_SESSION_CONTROL2, &mut named.0)?;
            let mut pid: u32 = 0;
            // S_FALSE across processes: not checked.
            method::<unsafe extern "system" fn(Object, *mut u32) -> HRESULT>(named.0, PROCESS_ID)(
                named.0, &mut pid,
            );
            if (wanted != 0 && i64::from(pid) != wanted)
                || window::process_name(i64::from(pid)) != program
            {
                return Ok(None);
            }
            let mut volume = Held(null_mut());
            query(control.0, &ISIMPLE_AUDIO_VOLUME, &mut volume.0)?;
            let (mut level, mut muted): (f32, i32) = (0.0, 0);
            checked(method::<
                unsafe extern "system" fn(Object, *mut f32) -> HRESULT,
            >(volume.0, GET_VOLUME)(volume.0, &mut level))?;
            checked(method::<
                unsafe extern "system" fn(Object, *mut i32) -> HRESULT,
            >(volume.0, GET_MUTE)(volume.0, &mut muted))?;
            Ok(Some(if muted != 0 { 0.0 } else { level as f64 }))
        }
    }
}

#[cfg(not(windows))]
mod com {
    pub struct Failed;

    pub fn slider(_program: &str, _wanted: i64) -> Result<Option<f64>, Failed> {
        Err(Failed)
    }
}
