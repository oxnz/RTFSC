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

fn set_fl(fd: libc::c_int, flags: libc::c_int) {
    unsafe {
        let mut v = libc::fcntl(fd, libc::F_GETFL);
        v |= flags;
        let r = libc::fcntl(fd, libc::F_SETFL, &v);
        assert_eq!(r, 0);
    }
}

fn clr_fl(fd: libc::c_int, flags: libc::c_int) {
    unsafe {
        let mut v = libc::fcntl(fd, libc::F_GETFL);
        v ^= flags;
        let r = libc::fcntl(fd, libc::F_SETFL, &v);
        assert_eq!(r, 0);
    }
}
