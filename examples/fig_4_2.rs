use std::{collections::VecDeque, ffi::CStr, mem::MaybeUninit, process::ExitCode};

use libc::{c_char, closedir, perror};

fn main() -> Result<(), ExitCode> {
    let mut argv = std::env::args().collect::<Vec<_>>();
    if argv.len() != 2 {
        eprintln!("usage: {} <path>", argv[0]);
        return Err(ExitCode::FAILURE);
    }

    let mut paths = VecDeque::new();
    paths.push_back(argv.pop().unwrap());

    while let Some(path) = paths.pop_front() {
        let mut stat: MaybeUninit<libc::stat> = MaybeUninit::uninit();
        if unsafe { libc::lstat(path.as_ptr() as *const c_char, stat.as_mut_ptr()) } < 0 {
            unsafe { perror("lstat".as_ptr() as *const _) };
            continue;
        }
        let stat = unsafe { stat.assume_init() };
        print!(
            "dev={}/{} rdev={}/{}",
            libc::major(stat.st_dev),
            libc::minor(stat.st_dev),
            libc::major(stat.st_rdev),
            libc::minor(stat.st_rdev)
        );
        match stat.st_mode & libc::S_IFMT {
            libc::S_IFBLK => {
                print!("b");
            }
            libc::S_IFCHR => {
                print!("c");
            }
            libc::S_IFDIR => unsafe {
                let dir = libc::opendir(path.as_ptr() as *const c_char);
                while let Some(dirent) = libc::readdir(dir).as_ref() {
                    match CStr::from_ptr(dirent.d_name.as_ptr()).to_str() {
                        Ok(name) => match name.as_ref() {
                            "." | ".." => continue,
                            _ => {
                                paths.push_back(format!("{path}/{name}"));
                            }
                        },
                        Err(e) => {
                            eprintln!("invalid name: {e:?}");
                            continue;
                        }
                    }
                }
                closedir(dir);
                print!("d");
            },
            libc::S_IFIFO => {
                print!("p");
            }
            libc::S_IFLNK => {
                print!("l");
            }
            libc::S_IFREG => {
                print!("-");
            }
            libc::S_IFSOCK => {
                print!("s");
            }
            _ => {}
        }
        println!("@{path}");
    }

    Ok(())
}
