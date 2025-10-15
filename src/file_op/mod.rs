use std::{ffi::CStr, mem::MaybeUninit};


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

pub struct DirReader<'a> {
    ptr: &'a mut libc::DIR,
}

impl<'a> TryFrom<&CStr> for DirReader<'a> {
    type Error = std::io::Error;

    fn try_from(value: &CStr) -> Result<Self, Self::Error> {
        if let Some(ptr) = unsafe { libc::opendir(value.as_ptr()).as_mut() } {
            Ok(Self { ptr })
        } else {
            Err(std::io::Error::last_os_error())
        }
    }
}

impl<'a> Drop for DirReader<'a> {
    fn drop(&mut self) {
        unsafe { libc::closedir(self.ptr) };
    }
}

impl<'a> Iterator for DirReader<'a> {
    type Item = &'a CStr;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(dirent) = unsafe { libc::readdir(self.ptr).as_ref() } {
            Some(unsafe { CStr::from_ptr(dirent.d_name.as_ptr()) })
        } else {
            None
        }
    }
}

pub struct Stat {
    stat: libc::stat,
    /*
         struct stat { /* when _DARWIN_FEATURE_64_BIT_INODE is NOT defined */
            mode_t   st_mode;   /* inode protection mode */
            struct timespec st_atimespec;  /* time of last access */
            struct timespec st_mtimespec;  /* time of last data modification */
            struct timespec st_ctimespec;  /* time of last file status change */
            quad_t   st_blocks; /* blocks allocated for file */
            u_long   st_blksize;/* optimal file sys I/O ops blocksize */
            u_long   st_flags;  /* user defined flags for file */
            u_long   st_gen;    /* file generation number */
        };
    */
}

impl TryFrom<&CStr> for Stat {
    type Error = std::io::Error;

    fn try_from(value: &CStr) -> Result<Self, Self::Error> {
        let mut stat: MaybeUninit<libc::stat> = MaybeUninit::uninit();
        if unsafe { libc::lstat(value.as_ptr(), stat.as_mut_ptr()) } < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            let stat = unsafe { stat.assume_init() };
            Ok(Self { stat })
        }
    }
}

impl Stat {
    pub fn is_dir(&self) -> bool {
        self.stat.st_mode & libc::S_IFMT == libc::S_IFDIR
    }
}

impl std::fmt::Debug for Stat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt = match self.stat.st_mode & libc::S_IFMT {
            libc::S_IFBLK => "b",
            libc::S_IFCHR => "c",
            libc::S_IFDIR => "d",
            libc::S_IFIFO => "p",
            libc::S_IFLNK => "l",
            libc::S_IFREG => "-",
            libc::S_IFSOCK => "s",
            _ => "?",
        };
        let dev_major = libc::major(self.stat.st_dev);
        let dev_minor = libc::minor(self.stat.st_dev);
        let rdev_major = libc::major(self.stat.st_rdev);
        let rdev_minor = libc::minor(self.stat.st_rdev);
        f.debug_struct("Stat")
            .field("ino", &self.stat.st_ino)
            .field("fmt", &fmt)
            .field("nlink", &self.stat.st_nlink)
            .field("size", &self.stat.st_size)
            .field("uid", &self.stat.st_uid)
            .field("gid", &self.stat.st_gid)
            .field("dev", &format!("{dev_major}/{dev_minor}"))
            .field("rdev", &format!("{rdev_major}/{rdev_minor}"))
            .finish()
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
