use std::mem::MaybeUninit;

fn main() {
    let fd = libc::STDIN_FILENO;
    if unsafe { libc::isatty(fd) } == 0 {
        println!("not a tty");
        return;
    }
    let vdisable = unsafe { libc::fpathconf(fd, libc::_PC_VDISABLE) };
    if vdisable < 0 {
        println!("fpathconf error");
    }
    let mut termios = unsafe {
        let mut p = MaybeUninit::uninit();
        libc::tcgetattr(fd, p.as_mut_ptr());
        p.assume_init()
    };
    termios.c_cc[libc::VINTR] = vdisable as u8; // disable INTR character
    termios.c_cc[libc::VEOF] = 2; // EOF is Ctrl-B
    let r = unsafe { libc::tcsetattr(fd, libc::TCSAFLUSH, &termios) };
    assert!(r == 0);
    unsafe { libc::pause() };
}
