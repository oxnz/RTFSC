use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    os::fd::FromRawFd,
    process::ExitCode,
};

// send data from parent to child over a pipe
fn main() -> Result<(), ExitCode> {
    let mut fds = [0, 0];
    if unsafe { libc::pipe(fds.as_mut_ptr()) } < 0 {
        unsafe { libc::perror(c"pipe".as_ptr()) };
        return Err(ExitCode::FAILURE);
    }

    let pid = unsafe { libc::fork() };
    let msg = "hello world";
    if pid < 0 {
        unsafe {
            libc::perror(c"fork".as_ptr());
        }
        return Err(ExitCode::FAILURE);
    } else if pid > 0 {
        // parent
        unsafe { libc::close(fds[0]) };
        let mut f = unsafe { File::from_raw_fd(fds[1]) };
        writeln!(f, "{}", msg).unwrap();
        writeln!(f, "bye").unwrap();
        drop(f);
        unsafe { libc::waitpid(pid, std::ptr::null_mut(), 0) };
    } else {
        // child
        unsafe { libc::close(fds[1]) };
        let f = unsafe { File::from_raw_fd(fds[0]) };
        for line in BufReader::new(f).lines() {
            match line {
                Ok(msg) => println!("rcvd: [{msg}]"),
                Err(_) => todo!(),
            }
        }
    }

    Ok(())
}
