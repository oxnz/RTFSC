use std::ffi::CStr;

pub struct Group(libc::group);

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

impl std::fmt::Debug for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = unsafe { CStr::from_ptr(self.0.gr_name) }.to_str().unwrap();
        f.debug_struct("Group")
            .field("gid", &self.0.gr_gid)
            .field("name", &name)
            .finish()
    }
}

impl std::fmt::Display for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = unsafe { CStr::from_ptr(self.0.gr_name) };
        write!(f, "{:?}", name)
    }
}
