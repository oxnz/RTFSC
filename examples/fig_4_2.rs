use std::{collections::VecDeque, ffi::CString, process::ExitCode, str::FromStr};

use rtfsc::file_op::{DirReader, Stat};

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
                println!("{stat:?} {path:?}");
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
