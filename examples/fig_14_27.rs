use std::{os::fd::AsRawFd, process::ExitCode};

use rtfsc::{mmap, munmap};

fn main() -> Result<(), ExitCode> {
    let argv = std::env::args().collect::<Vec<_>>();
    if argv.len() != 3 {
        eprintln!("usage: {} <from_file> <to_file>", argv[0]);
        return Err(ExitCode::FAILURE);
    }

    let fin = std::fs::OpenOptions::new()
        .read(true)
        .open(&argv[1])
        .unwrap();
    let len = fin.metadata().unwrap().len();
    let fd_in = fin.as_raw_fd();
    let fout = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&argv[2])
        .unwrap();
    fout.set_len(len).unwrap();
    let fd_out = fout.as_raw_fd();

    let mut offset: u64 = 0;
    let chunk_size = 1 << 30; // 1GB
    let total_len = len;
    while offset < total_len {
        let len = chunk_size.min(total_len - offset) as usize;
        let src = mmap(
            std::ptr::null_mut(),
            len,
            libc::PROT_READ,
            libc::MAP_SHARED,
            fd_in,
            offset as i64,
        )
        .unwrap();
        let dst = mmap(
            std::ptr::null_mut(),
            len,
            libc::PROT_WRITE,
            libc::MAP_SHARED,
            fd_out,
            offset as i64,
        )
        .unwrap();

        unsafe { std::ptr::copy_nonoverlapping(src, dst, len) };
        munmap(src, len).unwrap();
        munmap(dst, len).unwrap();
        offset += len as u64;
    }

    Ok(())
}
