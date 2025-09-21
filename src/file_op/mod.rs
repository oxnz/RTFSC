#[cfg(test)]
mod tests {
    #[test]
    fn test_seek() {
        assert_eq!(-1, unsafe {
            libc::lseek(libc::STDIN_FILENO, 0, libc::SEEK_SET)
        });
    }
}
