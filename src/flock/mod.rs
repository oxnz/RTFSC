use std::io::SeekFrom;

pub struct Flock {
    store: libc::flock,
}

impl Flock {
    fn new(op: i16, whence: SeekFrom, len: usize) -> Self {
        let mut store: libc::flock = unsafe { std::mem::zeroed() };

        store.l_type = op;
        match whence {
            SeekFrom::Start(offset) => {
                store.l_whence = libc::SEEK_SET as i16;
                store.l_start = offset as i64;
            }
            SeekFrom::End(offset) => {
                store.l_whence = libc::SEEK_END as i16;
                store.l_start = offset;
            }
            SeekFrom::Current(offset) => {
                store.l_whence = libc::SEEK_CUR as i16;
                store.l_start = offset;
            }
        }
        store.l_len = len as i64;
        Self { store }
    }

    pub fn read(whence: SeekFrom, len: usize) -> Self {
        Self::new(libc::F_RDLCK, whence, len)
    }

    pub fn write(whence: SeekFrom, len: usize) -> Self {
        Self::new(libc::F_WRLCK, whence, len)
    }

    pub fn unlock(whence: SeekFrom, len: usize) -> Self {
        Self::new(libc::F_UNLCK, whence, len)
    }

    pub fn set(&self, fd: i32) -> i32 {
        unsafe { libc::fcntl(fd, libc::F_SETLK, &self.store) }
    }

    pub fn set_wait(&self, fd: i32) -> i32 {
        unsafe { libc::fcntl(fd, libc::F_SETLKW, &self.store) }
    }

    pub fn test(&mut self, fd: i32) -> i32 {
        let r = unsafe { libc::fcntl(fd, libc::F_GETLK, &mut self.store) };
        if r < 0 {
            panic!("fcntl");
        }
        if self.store.l_type == libc::F_UNLCK {
            return 0;
        }
        return self.store.l_pid;
    }

    pub fn is_lockable(&mut self, fd: i32) -> bool {
        self.test(fd) == 0
    }
}
