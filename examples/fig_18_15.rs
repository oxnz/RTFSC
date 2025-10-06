use std::{mem::MaybeUninit, os::unix::ffi::OsStrExt, path::PathBuf};

fn main() {
    for fd in [libc::STDIN_FILENO, libc::STDERR_FILENO, libc::STDOUT_FILENO] {
        let name = ttyname(fd).unwrap();
        println!("fd {fd}: {name}");
    }
}

fn ttyname(fd: libc::c_int) -> std::io::Result<String> {
    let fstat = {
        let mut fstat = MaybeUninit::uninit();
        unsafe { libc::fstat(fd, fstat.as_mut_ptr()) };
        unsafe { fstat.assume_init() }
    };
    let mut paths: Vec<PathBuf> = vec![PathBuf::from("/dev")];
    while let Some(path) = paths.pop() {
        if let Ok(dir) = std::fs::read_dir(path) {
            for ent in dir {
                let p = ent?.path();
                let name = p.to_str().unwrap();
                let metadata = std::fs::metadata(&p).unwrap();
                if metadata.is_dir() {
                    paths.push(p);
                    continue;
                }
                let stat = {
                    let c_path = std::ffi::CString::new(p.as_os_str().as_bytes()).expect("cstr");
                    let mut stat = MaybeUninit::<libc::stat>::uninit();
                    if unsafe { libc::stat(c_path.as_ptr(), stat.as_mut_ptr()) } < 0 {
                        eprintln!("error");
                        continue;
                    }
                    unsafe { stat.assume_init() }
                };
                match name {
                    "/dev/stdin" | "/dev/stdout" | "/dev/stderr" => {
                        // skip aliases
                    }
                    _ => {
                        if stat.st_ino == fstat.st_ino && stat.st_dev == fstat.st_dev {
                            return Ok(name.to_string());
                        }
                    }
                }
            }
        }
    }
    Ok("NULL".to_string())
}
