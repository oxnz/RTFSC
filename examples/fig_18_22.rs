use std::mem::MaybeUninit;

fn print_winsz(fd: libc::c_int) {
    let sz = {
        let mut p: MaybeUninit<libc::winsize> = MaybeUninit::uninit();
        let r = unsafe { libc::ioctl(fd, libc::TIOCGWINSZ, p.as_mut_ptr()) };
        assert_ne!(r, -1);
        unsafe { p.assume_init() }
    };
    println!("{:?}", sz);
}

extern "C" fn on_sig_winch(signo: i32) {
    println!("on SIGWINCH");
    print_winsz(libc::STDIN_FILENO);
}

fn main() {
    unsafe {
        assert_eq!(libc::isatty(libc::STDIN_FILENO), 1, "tty expected");
        let r = libc::signal(libc::SIGWINCH, on_sig_winch as libc::sighandler_t);
        assert_ne!(r, libc::SIG_ERR);
    }
    print_winsz(libc::STDIN_FILENO);
    loop {
        unsafe { libc::pause() };
    }
}
