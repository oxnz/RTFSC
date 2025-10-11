use std::mem::MaybeUninit;

fn main() {
    let mut quitflag = false;
    let quitflag_ptr: &'static bool = unsafe { std::mem::transmute(&quitflag) };
    let mut mutex = Mutex::new();
    let mutex_ptr: &'static Mutex = unsafe { std::mem::transmute(&mutex) };
    let mut cond = libc::PTHREAD_COND_INITIALIZER;
    let cond_ptr: &'static libc::pthread_cond_t = unsafe { std::mem::transmute(&cond) };
    let mut mask = SigSet::empty();
    mask.add(libc::SIGINT);
    mask.add(libc::SIGQUIT);
    mask.block();
    let orig_sigset = mask.clone();
    let thread = std::thread::spawn(move || {
        loop {
            let signo = mask.wait();
            match signo {
                libc::SIGINT => {
                    println!("interrupt");
                }
                libc::SIGQUIT => unsafe {
                    println!("quit");
                    mutex_ptr.lock();
                    (quitflag_ptr as *const bool as *mut bool).write(true);
                    mutex_ptr.unlock();
                    libc::pthread_cond_signal(cond_ptr as *const libc::pthread_cond_t as *mut libc::pthread_cond_t);
                    break;
                },
                _ => {
                    panic!("unexpected signal: {signo}");
                }
            }
        }
    });
    unsafe {
        mutex.lock();
        while !quitflag {
            libc::pthread_cond_wait(&mut cond, &mut mutex.0);
        }
        println!("end lock");
        mutex.unlock();
    }
    // SIGQUIT has been caught and is now blocked; do whatever
    quitflag = false;

    thread.join().unwrap();
    // reset signal mask which unblocks SIGQUIT
    orig_sigset.mask();
}

struct Mutex(libc::pthread_mutex_t);

impl Mutex {
    const fn new() -> Self {
        Self(libc::PTHREAD_MUTEX_INITIALIZER)
    }

    fn lock(&self) {
        unsafe { libc::pthread_mutex_lock(self as *const _ as *mut _) };
    }

    fn unlock(&self) {
        unsafe { libc::pthread_mutex_unlock(self as *const _ as *mut _) };
    }
}

#[derive(Debug, Clone)]
struct SigSet {
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
