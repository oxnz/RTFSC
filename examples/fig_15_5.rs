use std::process::ExitCode;

fn main() -> Result<(), ExitCode> {
    let mut fds = [0, 0];
    if unsafe { libc::pipe(fds.as_mut_ptr()) } < 0 {
        unsafe { libc::perror("pipe".as_ptr() as *const _) };
        return Err(ExitCode::FAILURE);
    }

    let pid = unsafe { libc::fork() };
    let msg = b"hello world\n";
    if pid < 0 {
        unsafe {
            libc::perror("fork".as_ptr() as *const _);
        }
        return Err(ExitCode::FAILURE);
    } else if pid > 0 {
        // parent
        unsafe { libc::close(fds[0]) };
        unsafe { libc::write(fds[1], msg.as_ptr() as *const _, msg.len()) };
    } else {
        // child
        unsafe { libc::close(fds[1]) };
        let mut buf = vec![0; 128];
        let n = unsafe { libc::read(fds[0], buf.as_mut_ptr() as *mut _, buf.len()) };
        assert_eq!(n, msg.len() as isize);
        assert_eq!(&buf[0..msg.len()], msg);
    }

    Ok(())
}
