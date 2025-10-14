use rtfsc::file_op::{clr_fl, set_fl};

fn main() {
    let mut buf = [0; 500000];
    let fd = libc::STDOUT_FILENO;
    set_fl(fd, libc::O_NONBLOCK);
    let n = unsafe { libc::read(libc::STDIN_FILENO, buf.as_mut_ptr().cast(), buf.len()) };
    assert!(n >= 0);
    let mut buf = &buf[0..n as usize];
    while !buf.is_empty() {
        let n = unsafe { libc::write(fd, buf.as_ptr().cast(), buf.len()) };
        if n > 0 {
            buf = &buf[n as usize..];
            eprintln!("n = {n}");
        } else {
            eprintln!("n = {n}, error: {:?}", std::io::Error::last_os_error());
        }
    }
    clr_fl(fd, libc::O_NONBLOCK);
}
