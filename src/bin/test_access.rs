use std::process::ExitCode;

use libc::{O_RDONLY, perror};

fn main() -> Result<(), ExitCode> {
    let argv = std::env::args().collect::<Vec<_>>();
    if argv.len() != 2 {
        eprintln!("usage: {} <pathname>", argv[0]);
        return Err(ExitCode::FAILURE);
    }

    let path = argv[1].as_ptr() as *const _;

    if unsafe { libc::access(path, libc::R_OK) } < 0 {
        unsafe { perror("access".as_ptr() as *const _) };
    } else {
        println!("have read access");
    }

    let fd = unsafe { libc::open(path, O_RDONLY) };
    if fd < 0 {
        unsafe { perror("open".as_ptr() as *const _) };
    } else {
        println!("can open for reading");
        unsafe { libc::close(fd) };
    }

    Ok(())
}
