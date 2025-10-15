use std::ffi::CStr;

pub struct Passwd(libc::passwd);

impl From<libc::passwd> for Passwd {
    fn from(value: libc::passwd) -> Self {
        Self(value)
    }
}

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

impl std::fmt::Debug for Passwd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let uid = self.0.pw_uid;
        let gid = self.0.pw_gid;
        let name = unsafe { CStr::from_ptr(self.0.pw_name) }.to_str().unwrap();
        let shell = unsafe { CStr::from_ptr(self.0.pw_shell) }.to_str().unwrap();
        f.debug_struct("Passwd")
            .field("uid", &uid)
            .field("gid", &gid)
            .field("name", &name)
            .field("shell", &shell)
            .finish()
    }
}

impl std::fmt::Display for Passwd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = unsafe { CStr::from_ptr(self.0.pw_name) };
        write!(f, "{:?}", name)
    }
}

#[derive(Default)]
pub struct PasswdReader;

impl Iterator for PasswdReader {
    type Item = Passwd;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = unsafe { libc::getpwent().as_ref() } {
            Some(Passwd::from(*item))
        } else {
            unsafe { libc::endpwent() };
            None
        }
    }
}
