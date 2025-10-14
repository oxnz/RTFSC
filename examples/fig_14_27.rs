use std::{os::fd::AsRawFd, process::ExitCode};

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
        let len =  chunk_size.min(total_len - offset) as usize;
        let src = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                len,
                libc::PROT_READ,
                libc::MAP_SHARED,
                fd_in,
                offset as i64,
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
                offset as i64,
            )
        };
        if src == libc::MAP_FAILED {
            unsafe { libc::perror("mmap".as_ptr() as *const _) };
            return Err(ExitCode::FAILURE);
        }

        unsafe { libc::memcpy(dst, src, len) };
        unsafe { libc::munmap(src, len) };
        unsafe { libc::munmap(dst, len) };
        offset += len as u64;
    }

    Ok(())
}
