use std::{ffi::c_void, mem::MaybeUninit};

fn getpass(prompt: &str) -> String {
    let max_pass_len = 8;
    let mut buf = Vec::new();
    // libc::ctermid();
    let fd = unsafe { libc::open(c"/dev/tty".as_ptr() as *const _, libc::O_RDWR) };
    assert!(fd >= 0);

    let mut sigset = MaybeUninit::uninit();
    let mut orig_sigset = MaybeUninit::uninit();
    unsafe {
        libc::sigemptyset(sigset.as_mut_ptr());
        libc::sigaddset(sigset.as_mut_ptr(), libc::SIGINT); // block SIGINT
        libc::sigaddset(sigset.as_mut_ptr(), libc::SIGTSTP); // block SIGTSTP
        libc::sigprocmask(libc::SIG_BLOCK, sigset.as_ptr(), orig_sigset.as_mut_ptr());

        let mut termios = {
            let mut p = MaybeUninit::uninit();
            libc::tcgetattr(fd, p.as_mut_ptr());
            p.assume_init()
        };
        let orig_termios = termios.clone();
        termios.c_lflag &= !(libc::ECHO | libc::ECHOE | libc::ECHOK | libc::ECHONL);
        libc::tcsetattr(fd, libc::TCSAFLUSH, &termios);
        libc::write(fd, prompt.as_ptr() as *const _, prompt.len());

        let mut c: u8 = 0;
        loop {
            let r = libc::read(fd, &mut c as *mut u8 as *mut c_void, 1);
            if r != 1 || c as i32 == libc::EOF || c == b'\n' {
                break;
            }
            if buf.len() < max_pass_len {
                buf.push(c);
            }
        }
        buf.push(b'\0'); // null terminate
        libc::write(fd, c"\n".as_ptr().cast(), 1); // echo a newline

        libc::tcsetattr(fd, libc::TCSAFLUSH, &orig_termios); // restore TTY state
        libc::sigprocmask(
            libc::SIG_SETMASK,
            orig_sigset.as_ptr(),
            std::ptr::null_mut(),
        );
        libc::close(fd);
        String::from_utf8(buf).unwrap()
    }
}

fn main() {
    let mut pass = getpass("enter password:");
    println!("password: [{pass}]");
    let p = pass.as_mut_ptr();

    /* now use password (probably encrypt it)... */

    // zero it out when we're done with it
    for i in 0..pass.len() {
        unsafe { p.add(i).write(b'\0') };
    }
}
