use std::{
    io::{BufRead, Read},
    process::ExitCode,
};

use libc::{c_char, fork};

fn main() -> Result<(), ExitCode> {
    let argv = std::env::args().collect::<Vec<_>>();
    if argv.len() != 2 {
        eprintln!("usage: {} <pathname>", argv[0]);
        return Err(ExitCode::FAILURE);
    }

    let mut fds = [0; 2];

    unsafe { libc::pipe(fds.as_mut_ptr()) };

    let pid = unsafe { fork() };

    if pid < 0 {
        eprintln!("fork");
        return Err(ExitCode::FAILURE);
    } else if pid > 0 {
        // parent
        unsafe {
            libc::close(fds[0]);
            let fd = libc::open(argv[1].as_ptr() as *const _, libc::O_RDONLY);
            if fd < 0 {
                libc::perror("open\0".as_ptr() as *const _);
                return Err(ExitCode::FAILURE);
            }
            let mut file = std::fs::File::open(&argv[1]).unwrap();
            let mut buf = [0; 10];
            loop {
                match file.read(&mut buf[..]) {
                    Ok(n) => {
                        if n == 0 {
                            break;
                        }
                        if libc::write(fds[1], buf.as_ptr() as *const _, n) < 0 {
                            libc::perror("write\0".as_ptr() as *const _);
                            return Err(ExitCode::FAILURE);
                        }
                    }
                    Err(e) => {
                        eprintln!("read: {e:?}");
                        return Err(ExitCode::FAILURE);
                    }
                }
            }
            libc::close(fds[1]);
            if libc::waitpid(pid, std::ptr::null_mut(), 0) < 0 {
                libc::perror("waitpid\0".as_ptr() as *const _);
                return Err(ExitCode::FAILURE);
            }
        }
    } else {
        // child
        unsafe {
            libc::close(fds[1]);
            if libc::STDIN_FILENO != libc::dup2(fds[0], libc::STDIN_FILENO) {
                libc::perror("dup2\0".as_ptr() as *const _);
                return Err(ExitCode::FAILURE);
            }
            if libc::execl(
                "/usr/bin/more\0".as_ptr() as *const _,
                "more\0".as_ptr() as *const _,
                std::ptr::null() as *const c_char,
            ) < 0
            {
                libc::perror("execl\0".as_ptr() as *const _);
                return Err(ExitCode::FAILURE);
            }
        }
    }

    Ok(())
}
