use std::{ffi::CString, process::ExitCode, str::FromStr};

fn main() -> Result<(), ExitCode> {
    let argv = std::env::args().collect::<Vec<_>>();
    if argv.len() != 2 {
        eprintln!("usage: {} /proc/PID/fd/FD", argv[0]);
        return Err(ExitCode::FAILURE);
    }

    let path = CString::from_str(&argv[1]).unwrap();
    let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
    assert_ne!(-1, fd);

    let seals = unsafe { libc::fcntl(fd, libc::F_GET_SEALS) };
    assert_ne!(-1, seals);

    println!("existing seals:");
    if seals & libc::F_SEAL_SEAL != 0 {
        println!("- F_SEAL_SEAL");
    }
    if seals & libc::F_SEAL_GROW != 0 {
        println!("- F_SEAL_GROW");
    }
    if seals & libc::F_SEAL_WRITE != 0 {
        println!("- F_SEAL_WRITE");
    }
    if seals & libc::F_SEAL_FUTURE_WRITE != 0 {
        println!("- F_SEAL_FUTURE_WRITE");
    }
    if seals & libc::F_SEAL_SHRINK != 0 {
        println!("- F_SEAL_SHRINK");
    }

    // code to map the file and access the contents of the resulting mapping omitted

    Ok(())
}
