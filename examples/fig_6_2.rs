use std::ffi::CStr;

fn main() {
    let mut iter = PwIter::default();
    for item in iter {
        println!("{item:?}");
    }
}

#[derive(Debug)]
struct Password {
    uid: u32,
    gid: u32,
    name: String,
    shell: String,
}

impl From<&libc::passwd> for Password {
    fn from(value: &libc::passwd) -> Self {
        let name = unsafe { CStr::from_ptr(value.pw_name).to_str().unwrap().to_string() };
        let shell = unsafe { CStr::from_ptr(value.pw_shell) }
            .to_str()
            .unwrap()
            .to_string();
        Self {
            uid: value.pw_uid,
            gid: value.pw_gid,
            name,
            shell,
        }
    }
}

#[derive(Default)]
struct PwIter;

impl Iterator for PwIter {
    type Item = Password;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = unsafe { libc::getpwent().as_ref() } {
            Some(item.into())
        } else {
            unsafe { libc::endpwent() };
            None
        }
    }
}
