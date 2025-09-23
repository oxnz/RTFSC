use std::{ffi::CString, process::ExitCode, str::FromStr};

fn main() -> Result<(), ExitCode> {
    let argv = std::env::args().collect::<Vec<_>>();
    if argv.len() < 3 {
        eprintln!("usage: {} name size [seals]", argv[0]);
        eprintln!("\t'seals' can contain any of the following characters:");
        eprintln!("\t\tg - F_SEAL_GROW");
        eprintln!("\t\ts - F_SEAL_SHRINK");
        eprintln!("\t\tw - F_SEAL_WRITE");
        eprintln!("\t\tW - F_SEAL_FUTURE_WRITE");
        eprintln!("\t\tS - F_SEAL_SEAL");
        return Err(ExitCode::FAILURE);
    }

    let name = CString::from_str(&argv[1]).unwrap();
    let len: i64 = argv[2].parse().unwrap_or(10);
    let seal_arg = argv.get(3);

    // create an anonymous file in tmpfs; allow seals to be placed on the file.
    let fd = unsafe { libc::memfd_create(name.as_ptr(), libc::MFD_ALLOW_SEALING) };
    assert_ne!(fd, -1);

    // size the file as specified on the command line
    let r = unsafe { libc::ftruncate(fd, len) };
    assert_ne!(-1, r);

    let pid = unsafe { libc::getpid() };
    println!("PID: {} fd: {} /proc/{}/fd/{}", pid, fd, pid, fd);

    // code to map the file and populate the mapping with data omitted

    // if a 'seals' command line argument was supplied, set some seals on the file
    if let Some(s) = seal_arg {
        let mut seals = 0;
        for c in s.bytes() {
            match c {
                b'g' => {
                    seals |= libc::F_SEAL_GROW;
                }
                b's' => {
                    seals |= libc::F_SEAL_SHRINK;
                }
                b'w' => {
                    seals |= libc::F_SEAL_WRITE;
                }
                b'W' => {
                    seals |= libc::F_SEAL_FUTURE_WRITE;
                }
                b'S' => {
                    seals |= libc::F_SEAL_SEAL;
                }
                _ => {
                    eprintln!("unkonw seal: {c}, skipped");
                }
            }
        }

        let r = unsafe { libc::fcntl(fd, libc::F_ADD_SEALS, seals) };
        assert_ne!(-1, r);
    }

    // keep running, so that the file created by memfd_create() continues to exist.

    unsafe { libc::pause() };

    Ok(())
}
