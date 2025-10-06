use std::mem::MaybeUninit;

fn isatty(fd: libc::c_int) -> bool {
    let mut termios = MaybeUninit::uninit();
    unsafe { libc::tcgetattr(fd, termios.as_mut_ptr()) == 0 }
}

fn main() {
    for fd in [libc::STDIN_FILENO, libc::STDOUT_FILENO, libc::STDERR_FILENO] {
        println!("fd {fd} isatty: {}", isatty(fd));
    }
}
