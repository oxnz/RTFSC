use std::{ffi::c_void, mem::MaybeUninit};

extern "C" fn sig_catch(signo: libc::c_int) {
    println!("[signal] {signo}");
}

enum TtyState {
    Reset,
    Raw,
    CBreak,
}

/** put terminal into a cbreak mode */
fn tty_cbreak(fd: libc::c_int) {
    let mut buf = {
        let mut p = MaybeUninit::uninit();
        unsafe { libc::tcgetattr(fd, p.as_mut_ptr()) };
        unsafe { p.assume_init() }
    };
    // echo off, canonical mode off
    buf.c_lflag &= !(libc::ECHO | libc::ICANON);

    // case B: 1 byte at a time, no timer
    buf.c_cc[libc::VMIN] = 1;
    buf.c_cc[libc::VTIME] = 0;
    libc::tcsetattr(fd, libc::TCSAFLUSH, &buf);
}

fn tty_raw(fd: libc::c_int) {
    let mut buf = {
        let mut p = MaybeUninit::uninit();
        unsafe { libc::tcgetattr(fd, p.as_mut_ptr()) };
        unsafe { p.assume_init() }
    };
    // echo off, canonical mode off, extended input processing off, signal chars off
    buf.c_lflag &= !(libc::ECHO | libc::ICANON | libc::IEXTEN | libc::ISIG);

    /*
    - no sigint on break, CR-to-NL off, input parity check off,
    - doesn't strip 8th bit on input
    - output flow control off
     */
    buf.c_iflag &= !(libc::BRKINT | libc::ICRNL | libc::INPCK | libc::ISTRIP | libc::IXON);

    // clear size bits, parity checking off
    buf.c_cflag &= !(libc::CSIZE | libc::PARENB);

    // set 8 bits/char.
    buf.c_cflag |= libc::CS8;
}

fn tty_reset(fd: libc::c_int) {}

fn main() {
    let sig_handler = sig_catch as libc::sighandler_t;
    for sig in [libc::SIGINT, libc::SIGQUIT, libc::SIGQUIT] {
        unsafe { libc::signal(sig, sig_handler) };
    }

    tty_raw(libc::STDIN_FILENO);
    println!("enter raw mode characters, terminate with [DELETE]");
    let mut c: u8 = 0;
    while unsafe { libc::read(libc::STDIN_FILENO, &mut c as *mut u8 as *mut c_void, 1) } == 1 {
        c &= 255;
        if c == 0177 {
            // 0177 == ASCII DELETE
            break;
        }
        println!("{c:o}");
    }
    tty_reset(libc::STDIN_FILENO);
    tty_cbreak(libc::STDIN_FILENO);
    println!("enter cbreak mode characters, terminate with [SIGINT]");
    while unsafe { libc::read(libc::STDIN_FILENO, &mut c as *mut u8 as *mut c_void, 1) } == 1 {
        c &= 255;
        println!("{c:o}");
    }
    tty_reset(libc::STDIN_FILENO);
}
