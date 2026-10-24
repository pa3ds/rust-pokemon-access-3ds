//! One pa3ds at a time, and out of sight while Azahar is open (ADR-0039).
//!
//! A real process stands in for the pa3ds replaced and for Azahar: what is measured is whether it
//! is ended, and whether its end is noticed, which nothing short of a process answers.

use std::cell::{Cell, RefCell};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::core::{instance, toml};

/// pytest's `tmp_path`: a folder of this test's own, gone when the test is.
struct TmpPath(PathBuf);

impl TmpPath {
    fn new() -> Self {
        static MADE: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "pa3ds-test-instance-{}-{}",
            std::process::id(),
            MADE.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&path).unwrap();
        TmpPath(path)
    }
}

impl Drop for TmpPath {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

impl std::ops::Deref for TmpPath {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

/// A program that runs until it is ended: the Python's sleeping interpreter, as `ping`, which
/// needs no console to wait in.
struct Other(Child);

impl Other {
    fn new() -> Self {
        Other(
            Command::new("ping")
                .args(["-n", "61", "127.0.0.1"])
                .stdout(Stdio::null())
                .spawn()
                .unwrap(),
        )
    }

    fn pid(&self) -> i64 {
        i64::from(self.0.id())
    }
}

impl Drop for Other {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn lock(home: &Path, pid: i64, started: i64) {
    fs::create_dir_all(home).unwrap();
    fs::write(
        home.join(instance::LOCK),
        format!("pid = {pid}\nstarted = {started}\n"),
    )
    .unwrap();
}

/// With exit code 0, or Windows Terminal keeps a hidden window open for it.
#[test]
fn test_a_pa3ds_started_again_ends_the_one_running_and_takes_its_place() {
    let tmp_path = TmpPath::new();
    let mut other = Other::new();
    lock(&tmp_path, other.pid(), instance::started(other.pid()));
    assert!(instance::replace(&tmp_path).unwrap());
    assert_eq!(other.0.wait().unwrap().code(), Some(0));
    assert_eq!(
        toml::read(&tmp_path.join(instance::LOCK))["pid"],
        ::toml::Value::Integer(i64::from(std::process::id()))
    );
}

/// A process with the number the file names, started at another moment, is somebody else's
/// program that has the number since.
#[test]
fn test_a_file_left_by_a_pa3ds_that_died_ends_nothing() {
    let tmp_path = TmpPath::new();
    let mut other = Other::new();
    lock(&tmp_path, other.pid(), instance::started(other.pid()) + 1);
    assert!(!instance::replace(&tmp_path).unwrap());
    assert!(other.0.try_wait().unwrap().is_none());
}

#[test]
fn test_the_file_goes_as_pa3ds_closes_but_not_once_another_has_taken_its_place() {
    let tmp_path = TmpPath::new();
    instance::replace(&tmp_path).unwrap();
    instance::release(&tmp_path);
    assert!(!tmp_path.join(instance::LOCK).exists());
    lock(&tmp_path, i64::from(std::process::id()) + 4, 1);
    instance::release(&tmp_path);
    assert!(tmp_path.join(instance::LOCK).exists());
}

/// Before Azahar has been seen pa3ds stays in sight and open, as when it was started first.
#[test]
fn test_pa3ds_is_hidden_while_azahar_is_open_and_closes_once_it_has_closed() {
    let mut other = Other::new();
    let azahar = Rc::new(Cell::new(0));
    let hidden = Rc::new(Cell::new(false));
    let logged = Rc::new(RefCell::new(Vec::<String>::new()));
    let beside = {
        let (azahar, hidden, logged) = (azahar.clone(), hidden.clone(), logged.clone());
        instance::Beside::with(
            Rc::new(move |line: &str| logged.borrow_mut().push(line.to_string())),
            0.0,
            Rc::new(move || azahar.get()),
            Rc::new(move || hidden.set(true)),
        )
    };
    assert!(!beside.closed() && !hidden.get());
    azahar.set(other.pid());
    assert!(!beside.closed() && hidden.get());
    other.0.kill().unwrap();
    other.0.wait().unwrap();
    assert!(beside.closed());
    assert_eq!(
        *logged.borrow(),
        ["Azahar has closed, and pa3ds closes with it"]
    );
}

/// A double-clicked pa3ds's window stands in for its console, which is hidden from the start;
/// the window is hidden while Azahar is open, as the console was.
#[test]
fn test_a_pa3ds_with_its_own_window_hides_its_console_at_once_and_its_window_with_azahar() {
    let other = Other::new();
    let azahar = Rc::new(Cell::new(0));
    let consoles = Rc::new(Cell::new(false));
    let windows = Rc::new(Cell::new(false));
    let beside = {
        let (azahar, consoles, windows) = (azahar.clone(), consoles.clone(), windows.clone());
        instance::Beside::with(
            Rc::new(|_line: &str| {}),
            0.0,
            Rc::new(move || azahar.get()),
            Rc::new(move || consoles.set(true)),
        )
        .own_window(true, Some(Rc::new(move || windows.set(true))))
    };
    assert!(!beside.closed() && consoles.get() && !windows.get());
    azahar.set(other.pid());
    assert!(!beside.closed() && windows.get());
}
