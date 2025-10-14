pub fn set_fl(fd: libc::c_int, flags: libc::c_int) {
    unsafe {
        let mut v = libc::fcntl(fd, libc::F_GETFL);
        v |= flags;
        let r = libc::fcntl(fd, libc::F_SETFL, &v);
        assert_eq!(r, 0);
    }
}

pub fn clr_fl(fd: libc::c_int, flags: libc::c_int) {
    unsafe {
        let mut v = libc::fcntl(fd, libc::F_GETFL);
        v ^= flags;
        let r = libc::fcntl(fd, libc::F_SETFL, &v);
        assert_eq!(r, 0);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_seek() {
        assert_eq!(-1, unsafe {
            libc::lseek(libc::STDIN_FILENO, 0, libc::SEEK_SET)
        });
    }
}
