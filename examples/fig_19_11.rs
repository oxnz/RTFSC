use std::{
    ffi::{CStr, CString},
    mem::MaybeUninit,
};

use libc::{c_char, tcgetattr};

fn main() {
    let optstr = "+d:einv";
    let mut c = 0;
    let args: Vec<CString> = std::env::args().map(|s| CString::new(s).unwrap()).collect();
    let argc = args.len();
    let mut argv = args.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();
    argv.push(std::ptr::null());

    let mut driver: Option<String> = None;
    let mut noecho = false;
    let mut ignoreeof = false;
    let mut interactive = true;
    let mut verbose = false;

    while {
        c = unsafe {
            libc::getopt(
                argc as libc::c_int,
                argv.as_ptr() as *const _,
                optstr.as_ptr() as *const i8,
            )
        };
        c != -1
    } {
        match c as u8 {
            // driver for stdin/stdout
            b'd' => {
                let arg = unsafe { CStr::from_ptr(optarg) };
                driver = Some(arg.to_string_lossy().to_string())
            }
            // noecho for slave pty’s line discipline
            b'e' => {
                noecho = true;
            }
            // ignore EOF on standard input
            b'i' => {
                ignoreeof = true;
            }
            b'n' => {
                interactive = false;
            }
            b'v' => {
                verbose = true;
            }
            b'?' => {
                eprintln!("unrecognized option:");
            }
            _ => {
                println!("{c}");
            }
        }
    }

    if unsafe { optind } as usize >= argc {
        eprintln!(
            "usage: {:?} [ -d driver -einv ] program [ arg ... ]",
            args[0]
        );
        return;
    }

    let pid = if interactive {
        // fetch current termios and window size
        let mut termios = unsafe {
            let mut p = MaybeUninit::uninit();
            tcgetattr(libc::STDIN_FILENO, p.as_mut_ptr());
            p.assume_init()
        };
        let winsize = unsafe {
            let mut p: MaybeUninit<libc::winsize> = MaybeUninit::uninit();
            libc::ioctl(libc::STDIN_FILENO, libc::TIOCGWINSZ, p.as_mut_ptr());
            p.assume_init()
        };
        0
    } else {
        0
    };

    if pid < 0 {
        panic!("fork");
    } else if pid == 0 {
        if noecho {
            // stdin is slave pty
            // set_noecho(libc::STDIN_FILENO);
        }
        let r = unsafe { libc::execvp(argv[optind as usize], argv[optind as usize..].as_ptr()) };
        if r < 0 {
            panic!("execvp");
        }
    }

    if verbose {
        if let Some(d) = driver.as_ref() {
            println!("driver: {d}");
        }
    }

    if interactive && driver.is_none() {
        // tty_raw(libc::STDIN_FILENO);
        // atexit(tty_atexit);
    }

    if driver.is_some() {
        // do_driver(driver.take());
    }

    // loop_io(fdm, ignoreeof);
}

unsafe extern "C" {
    static optarg: *const c_char;
    static optind: libc::c_int;
}

struct PtyMaster {
    fd: libc::c_int,
    name: *mut libc::c_char,
}

impl PtyMaster {
    pub fn new() -> Self {
        unsafe {
            let fd = libc::posix_openpt(libc::O_RDWR);
            assert!(fd >= 0);
            // grant access to slave
            if libc::grantpt(fd) < 0 {
                panic!("grantpt: {:?}", std::io::Error::last_os_error());
            }
            // clear slave’s lock flag
            if libc::unlockpt(fd) < 0 {
                panic!("grantpt: {:?}", std::io::Error::last_os_error());
            }
            // get slave’s name
            let ptr = libc::ptsname(fd);
            Self { fd, name: ptr }
        }
    }
}

struct PytSlave {
    fd: libc::c_int,
}

impl PytSlave {
    pub fn new(path: *mut libc::c_char) -> Self {
        unsafe {
            let fd = libc::open(path, libc::O_RDWR);
            assert!(fd >= 0);
            Self { fd }
        }
    }
}

fn pyt_fork() -> libc::pid_t {
    let master = PtyMaster::new();
    let pid = unsafe { libc::fork() };
    if pid < 0 {
        panic!("fork: {:?}", std::io::Error::last_os_error());
    }
    if pid == 0 {
        // child
        unsafe { libc::setsid() };
        let slave = PytSlave::new(master.name);
        #[cfg(target_os = "BSD")]
        libc::ioctl(slave.fd, libc::TIOCSCTTY, std::ptr::null());

        unsafe {
            // libc::tcsetattr(slave.fd, libc::TCSANOW, slave_termios);
            // libc::ioctl(slave.fd, libc::TIOCSWINSZ, slave_winsize);
        }

        // Slave becomes stdin/stdout/stderr of child.
        for dst in [libc::STDIN_FILENO, libc::STDOUT_FILENO, libc::STDERR_FILENO] {
            assert_eq!(unsafe { libc::dup2(slave.fd, dst) }, dst);
        }
        return pid;
    } else {
        // parent
        return pid;
    }
}
