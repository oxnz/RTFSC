use std::{ffi::c_void, time::Duration};

#[derive(Debug)]
pub struct KQueue {
    fd: i32,
}

impl Drop for KQueue {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
    }
}

impl KQueue {
    pub fn try_new() -> std::io::Result<Self> {
        unsafe {
            let fd = libc::kqueue();
            if fd != -1 {
                Ok(Self { fd })
            } else {
                Err(std::io::Error::last_os_error())
            }
        }
    }

    pub fn update(&mut self, events: &[KEvent]) -> std::io::Result<()> {
        let n = unsafe {
            libc::kevent(
                self.fd,
                events.as_ptr().cast(),
                events.len() as i32,
                std::ptr::null_mut(),
                0,
                std::ptr::null(),
            )
        };
        match n.cmp(&0) {
            std::cmp::Ordering::Less => Err(std::io::Error::last_os_error()),
            std::cmp::Ordering::Equal => Ok(()),
            std::cmp::Ordering::Greater => todo!(),
        }
    }

    pub fn poll(
        &mut self,
        events: &mut [KEvent],
        timeout: Option<Duration>,
    ) -> std::io::Result<usize> {
        let timeout = timeout.map(|o| libc::timespec {
            tv_sec: o.as_secs().cast_signed(),
            tv_nsec: o.as_nanos() as i64,
        });
        let timeout_ptr = timeout.as_ref().map(|o| o as *const _).unwrap_or_default();
        let n = unsafe {
            libc::kevent(
                self.fd,
                std::ptr::null(),
                0,
                events.as_mut_ptr().cast(),
                events.len() as i32,
                timeout_ptr,
            )
        };
        match n.cmp(&0) {
            std::cmp::Ordering::Less => Err(std::io::Error::last_os_error()),
            std::cmp::Ordering::Equal => {
                Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout"))
            }
            std::cmp::Ordering::Greater => Ok(n as usize),
        }
    }
}

#[derive(Debug)]
pub struct KEvent(libc::kevent);

impl KEvent {
    pub fn new(ident: usize, action: Action, filter: Filter) -> Self {
        Self::raw(ident, filter.0, action.0, 0, 0, std::ptr::null_mut())
    }

    fn raw(
        ident: usize,
        filter: i16,
        flags: u16,
        fflags: u32,
        data: isize,
        udata: *mut c_void,
    ) -> Self {
        Self(libc::kevent {
            ident,
            filter,
            flags,
            fflags,
            data,
            udata,
        })
    }

    pub fn ident(&self) -> usize {
        self.0.ident
    }

    pub fn readable(&self) -> bool {
        self.0.filter == libc::EVFILT_READ
    }

    pub fn writable(&self) -> bool {
        self.0.filter == libc::EVFILT_WRITE
    }

    pub fn error(&self) -> bool {
        self.0.flags == libc::EV_ERROR
    }
}

#[derive(Debug)]
pub struct Action(u16);

impl Action {
    pub fn add() -> Self {
        Self(libc::EV_ADD | libc::EV_ENABLE)
    }

    pub fn remove() -> Self {
        Self(libc::EV_DELETE)
    }
}

#[derive(Debug)]
pub struct Filter(i16);

impl Filter {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn read(mut self) -> Self {
        self.0 |= libc::EVFILT_READ;
        self
    }

    pub fn write(mut self) -> Self {
        self.0 |= libc::EVFILT_WRITE;
        self
    }
}

#[test]
fn test_lifecycle() {
    let q = KQueue::try_new().unwrap();
    drop(q);
}

#[test]
fn test_register() {
    let mut q = KQueue::try_new().unwrap();
    let event = KEvent::new(
        libc::STDOUT_FILENO as usize,
        Action::add(),
        Filter::new().write(),
    );
    let mut events = vec![event];
    let r = q.update(&events);
    assert!(r.is_ok());
    let r = q.poll(&mut events, None);
    assert_eq!(r.ok(), Some(1));
}
