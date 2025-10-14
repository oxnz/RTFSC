use std::ffi::c_void;

pub mod file_op;
pub mod flock;
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
