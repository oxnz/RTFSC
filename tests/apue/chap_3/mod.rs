use libc::lseek;

#[test]
fn figure_3_2() {
    let buf1 = b"abcdefghij";
    let buf2 = b"ABCDEFGHIJ";

    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("file.hole");

    let fd = unsafe {
        libc::creat(
            path.to_str().unwrap().as_ptr() as _,
            libc::S_IRUSR | libc::S_IWUSR | libc::S_IRGRP | libc::S_IROTH,
        )
    };
    assert_ne!(-1, fd);

    assert_eq!(buf1.len() as isize, unsafe {
        libc::write(fd, buf1.as_ptr() as _, buf1.len())
    });

    if unsafe { lseek(fd, 16384, libc::SEEK_SET) } == -1 {
        unsafe { libc::perror(b"lseek".as_ptr() as _) };
    }

    assert_eq!(buf2.len() as isize, unsafe {
        libc::write(fd, buf2.as_ptr() as _, buf2.len())
    });

    unsafe { libc::close(fd) };

    let content = std::fs::read(path).unwrap();
    let expected = [
        buf1.as_slice(),
        b"\0".repeat(16384 - buf1.len()).as_slice(),
        buf2.as_slice(),
    ]
    .concat();
    assert_eq!(content, expected);
}
