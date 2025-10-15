use std::{
    collections::VecDeque,
    ffi::{CStr, CString},
    mem::MaybeUninit,
    process::ExitCode,
    str::FromStr,
};

use libc::perror;

fn main() -> Result<(), ExitCode> {
    let argv = std::env::args().collect::<Vec<_>>();
    if argv.len() != 2 {
        eprintln!("usage: {} <path>", argv[0]);
        return Err(ExitCode::FAILURE);
    }

    let mut paths = VecDeque::new();
    paths.push_back(CString::from_str(&argv[1]).unwrap());

    while let Some(path) = paths.pop_front() {
        match Stat::try_from(path.as_c_str()) {
            Ok(stat) => {
                println!("{stat} {path:?}");
                if stat.is_dir() {
                    for name in DirReader::try_from(path.as_c_str()).unwrap() {
                        match name.to_bytes() {
                            b"." | b".." => continue,
                            _name => {
                                let mut buf =
                                    Vec::with_capacity(path.count_bytes() + 1 + _name.len());
                                buf.extend_from_slice(path.as_bytes());
                                buf.push(b'/');
                                buf.extend_from_slice(_name);
                                let p = unsafe { CString::from_vec_unchecked(buf) };
                                paths.push_back(p);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("error: {e:?}");
            }
        }
    }

    Ok(())
}

struct DirReader<'a> {
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

struct Stat {
    stat: libc::stat,
    /*
         struct stat { /* when _DARWIN_FEATURE_64_BIT_INODE is NOT defined */
            ino_t    st_ino;    /* inode's number */
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

impl std::fmt::Display for Stat {
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
        write!(
            f,
            "{fmt} dev={}/{} rdev={}/{} nlink={} size={} user={} group={}",
            dev_major,
            dev_minor,
            rdev_major,
            rdev_minor,
            self.stat.st_nlink,
            self.stat.st_size,
            Passwd::try_from(self.stat.st_uid).unwrap(),
            Group::try_from(self.stat.st_gid).unwrap()
        )
    }
}

struct Passwd(libc::passwd);

impl TryFrom<u32> for Passwd {
    type Error = std::io::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if let Some(passwd) = unsafe { libc::getpwuid(value).as_ref() } {
            Ok(Self(*passwd))
        } else {
            Err(std::io::Error::last_os_error())
        }
    }
}

impl std::fmt::Display for Passwd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = unsafe { CStr::from_ptr(self.0.pw_name) };
        write!(f, "{:?}", name)
    }
}

struct Group(libc::group);

impl TryFrom<u32> for Group {
    type Error = std::io::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if let Some(group) = unsafe { libc::getgrgid(value).as_ref() } {
            Ok(Self(*group))
        } else {
            Err(std::io::Error::last_os_error())
        }
    }
}

impl std::fmt::Display for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = unsafe { CStr::from_ptr(self.0.gr_name) };
        write!(f, "{:?}", name)
    }
}
