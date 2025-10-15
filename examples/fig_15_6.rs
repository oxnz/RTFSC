use std::{
    fs::File,
    io::{Read, Write},
    os::fd::FromRawFd,
    process::ExitCode,
};

use rtfsc::{execvp, waitpid};

fn main() -> Result<(), ExitCode> {
    let argv = std::env::args().collect::<Vec<_>>();
    if argv.len() != 2 {
        eprintln!("usage: {} <pathname>", argv[0]);
        return Err(ExitCode::FAILURE);
    }

    let mut fds = [0; 2];

    unsafe { libc::pipe(fds.as_mut_ptr()) };

    let pid = unsafe { libc::fork() };

    if pid < 0 {
        eprintln!("fork");
        return Err(ExitCode::FAILURE);
    } else if pid > 0 {
        // parent
        unsafe {
            libc::close(fds[0]);

            let mut in_f = std::fs::File::open(&argv[1]).unwrap();
            let mut out_f = File::from_raw_fd(fds[1]);
            let mut buf = [0; 1024];
            loop {
                let n = in_f.read(&mut buf).unwrap();
                if n == 0 {
                    break;
                }
                out_f.write(&buf[..n]).unwrap();
            }
            drop(out_f);
            waitpid(pid, 0).unwrap();
        }
    } else {
        // child
        unsafe {
            libc::close(fds[1]);
            if fds[0] != libc::STDIN_FILENO {
                if libc::STDIN_FILENO != libc::dup2(fds[0], libc::STDIN_FILENO) {
                    libc::perror(c"dup2".as_ptr());
                    return Err(ExitCode::FAILURE);
                }
                libc::close(fds[0]);
            }
            execvp("more", &["more"]).unwrap();
        }
    }

    Ok(())
}
