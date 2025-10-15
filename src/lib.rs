use std::{
    ffi::{CString, c_void},
    mem::MaybeUninit,
    str::FromStr,
};

pub mod file_op;
pub mod flock;
pub mod group;
pub mod passwd;
pub mod signal;

pub fn mmap(
    addr: *mut c_void,
    len: usize,
    prot: i32,
    flags: i32,
    fd: i32,
    offset: i64,
) -> std::io::Result<*mut c_void> {
    let r = unsafe { libc::mmap(addr, len, prot, flags, fd, offset) };
    if libc::MAP_FAILED == r {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(r)
    }
}

pub fn munmap(addr: *mut c_void, len: usize) -> std::io::Result<()> {
    if -1 == unsafe { libc::munmap(addr, len) } {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub fn waitpid(pid: i32, options: i32) -> std::io::Result<i32> {
    let mut status = MaybeUninit::uninit();
    if unsafe { libc::waitpid(pid, status.as_mut_ptr(), options) } < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(unsafe { status.assume_init() })
    }
}

pub fn execvp(name: &str, argv: &[&str]) -> std::io::Result<()> {
    let s = CString::from_str(name).unwrap();
    let v = argv
        .into_iter()
        .map(|&s| CString::from_str(s).unwrap())
        .collect::<Vec<_>>();
    let mut ptrs = v.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();
    ptrs.push(std::ptr::null());
    if -1 == unsafe { libc::execvp(s.as_ptr(), ptrs.as_ptr()) } {
        Err(std::io::Error::last_os_error())
    } else {
        unreachable!("execvp")
    }
}
