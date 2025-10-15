use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    os::fd::FromRawFd,
};

fn main() -> std::io::Result<()> {
    unsafe { libc::signal(libc::SIGPIPE, on_sigpipe as usize) };
    let mut input = [0; 2];
    let mut output = [0; 2];
    unsafe {
        libc::pipe(input.as_mut_ptr());
        libc::pipe(output.as_mut_ptr());
    }
    let pid = unsafe { libc::fork() };
    if pid < 0 {
        return Err(std::io::Error::last_os_error());
    }
    if pid > 0 {
        unsafe {
            libc::close(input[0]);
            libc::close(output[1]);
            let mut f_in = File::from_raw_fd(input[1]);
            let mut f_out = BufReader::new(File::from_raw_fd(output[0]));
            for line in std::io::stdin().lines() {
                let mut line = line.unwrap();
                writeln!(f_in, "{}", line).unwrap();
                line.clear();
                f_out.read_line(&mut line).unwrap();
                print!("{line}");
            }
        }
    } else {
        unsafe {
            libc::close(input[1]);
            if input[0] != libc::STDIN_FILENO {
                libc::dup2(input[0], libc::STDIN_FILENO);
            }
            libc::close(output[0]);
            if output[1] != libc::STDOUT_FILENO {
                libc::dup2(output[1], libc::STDOUT_FILENO);
            }
        }
        if unsafe {
            libc::execl(
                c"/bin/cat".as_ptr(),
                c"cat".as_ptr(),
                std::ptr::null() as *const libc::c_char,
            )
        } < 0
        {
            panic!("execl");
        }
    }
    Ok(())
}

unsafe extern "C" fn on_sigpipe(signo: i32) {
    eprintln!("rcvd signal: {signo}");
    std::process::exit(1);
}
