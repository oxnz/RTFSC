use std::ffi::CStr;

/**
 * cargo run --package rtfsc --example fig_18_16  < /dev/console 2> /dev/null
 *
 * fd 0: /dev/console
 * fd 2: not a tty
 * fd 1: /dev/ttys008
 */

fn main() {
    for fd in [libc::STDIN_FILENO, libc::STDERR_FILENO, libc::STDOUT_FILENO] {
        let name = unsafe {
            let mut buf = [0; 128];
            if libc::isatty(fd) == 1 {
                libc::ttyname_r(fd, buf.as_mut_ptr(), buf.len());
                CStr::from_ptr(buf.as_ptr()).to_str().unwrap()
            } else {
                "not a tty"
            }
        };
        println!("fd {}: {}", fd, name);
    }
}
