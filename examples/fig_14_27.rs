use std::{mem::MaybeUninit, process::ExitCode};

use libc::{MAP_SHARED, c_char};

fn main() -> Result<(), ExitCode> {
    let argv = std::env::args().collect::<Vec<_>>();
    if argv.len() != 3 {
        eprintln!("usage: {} <from_file> <to_file>", argv[0]);
        return Err(ExitCode::FAILURE);
    }

    let fd_in = unsafe { libc::open(argv[1].as_ptr() as *const c_char, libc::O_RDONLY) };
    if fd_in < 0 {
        unsafe { libc::perror("open".as_ptr() as *const c_char) };
        return Err(ExitCode::FAILURE);
    }

    let fd_out = unsafe {
        libc::open(
            argv[2].as_ptr() as *const c_char,
            libc::O_RDWR | libc::O_CREAT | libc::O_TRUNC,
            (libc::S_IRUSR | libc::S_IWUSR | libc::S_IRGRP | libc::S_IROTH) as libc::c_uint,
        )
    };

    if fd_in < 0 {
        unsafe { libc::perror("open".as_ptr() as *const c_char) };
        return Err(ExitCode::FAILURE);
    }

    let stat = unsafe {
        let mut stat: MaybeUninit<libc::stat> = MaybeUninit::uninit();
        if libc::fstat(fd_in, stat.as_mut_ptr()) < 0 {
            libc::perror("fstat".as_ptr() as *const _);
            return Err(ExitCode::FAILURE);
        }
        stat.assume_init()
    };

    if unsafe { libc::ftruncate(fd_out, stat.st_size) } < 0 {
        unsafe { libc::perror("ftruncate".as_ptr() as *const _) };
        return Err(ExitCode::FAILURE);
    }

    let mut offset = 0;
    let chunk_size = 1 << 30; // 1GB
    while offset < stat.st_size {
        let len = if stat.st_size - offset > chunk_size {
            chunk_size
        } else {
            stat.st_size - offset
        } as usize;

        let src = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                len,
                libc::PROT_READ,
                MAP_SHARED,
                fd_in,
                offset,
            )
        };
        if src == libc::MAP_FAILED {
            unsafe { libc::perror("mmap".as_ptr() as *const _) };
            return Err(ExitCode::FAILURE);
        }
        let dst = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                len,
                libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd_out,
                offset,
            )
        };
        if src == libc::MAP_FAILED {
            unsafe { libc::perror("mmap".as_ptr() as *const _) };
            return Err(ExitCode::FAILURE);
        }

        unsafe { libc::memcpy(dst, src, len) };
        unsafe { libc::munmap(src, len) };
        unsafe { libc::munmap(dst, len) };
        offset += len as i64;
    }

    Ok(())
}
