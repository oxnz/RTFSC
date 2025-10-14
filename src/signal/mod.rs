use std::mem::MaybeUninit;

#[derive(Debug, Clone)]
pub struct SigSet {
    store: libc::sigset_t,
}

impl SigSet {
    pub fn empty() -> Self {
        let mut store = MaybeUninit::uninit();
        unsafe { libc::sigemptyset(store.as_mut_ptr()) };
        Self {
            store: unsafe { store.assume_init() },
        }
    }

    pub fn add(&mut self, signo: libc::c_int) {
        unsafe { libc::sigaddset(&mut self.store, signo) };
    }

    pub fn block(&self) -> SigSet {
        unsafe {
            let mut orig = MaybeUninit::uninit();
            libc::pthread_sigmask(libc::SIG_BLOCK, &self.store, orig.as_mut_ptr());
            SigSet {
                store: orig.assume_init(),
            }
        }
    }

    pub fn unblock(&self) -> SigSet {
        unsafe {
            let mut orig = MaybeUninit::uninit();
            libc::pthread_sigmask(libc::SIG_UNBLOCK, &self.store, orig.as_mut_ptr());
            SigSet {
                store: orig.assume_init(),
            }
        }
    }

    pub fn mask(&self) -> SigSet {
        unsafe {
            let mut orig = MaybeUninit::uninit();
            libc::pthread_sigmask(libc::SIG_SETMASK, &self.store, orig.as_mut_ptr());
            SigSet {
                store: orig.assume_init(),
            }
        }
    }

    pub fn wait(&self) -> libc::c_int {
        let mut signo = MaybeUninit::uninit();
        unsafe {
            libc::sigwait(&self.store, signo.as_mut_ptr());
            signo.assume_init()
        }
    }
}

impl From<&[libc::c_int]> for SigSet {
    fn from(value: &[libc::c_int]) -> Self {
        let mut o = Self::empty();
        for &signo in value {
            o.add(signo);
        }
        o
    }
}
