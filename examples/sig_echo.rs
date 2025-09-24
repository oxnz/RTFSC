use std::{ffi::c_void, mem::MaybeUninit, process::ExitCode};

fn main() -> Result<(), ExitCode> {
    let mask = {
        let mut mask: MaybeUninit<libc::sigset_t> = MaybeUninit::uninit();
        unsafe {
            libc::sigemptyset(mask.as_mut_ptr());
            libc::sigaddset(mask.as_mut_ptr(), libc::SIGINT);
            libc::sigaddset(mask.as_mut_ptr(), libc::SIGTERM);
            libc::sigaddset(mask.as_mut_ptr(), libc::SIGHUP);
            libc::sigprocmask(libc::SIG_BLOCK, mask.as_ptr(), std::ptr::null_mut());
            mask.assume_init()
        }
    };

    let fd = unsafe { libc::signalfd(-1, std::ptr::addr_of!(mask), 0) };
    assert!(fd > 0);

    loop {
        let si = {
            let mut si: MaybeUninit<libc::signalfd_siginfo> = MaybeUninit::uninit();
            unsafe {
                let n = libc::read(
                    fd,
                    si.as_mut_ptr() as *mut c_void,
                    size_of::<libc::signalfd_siginfo>(),
                );
                assert!(n > 0);
                assert_eq!(n as usize, size_of::<libc::signalfd_siginfo>());
                si.assume_init()
            }
        };
        match si.ssi_signo as i32 {
            libc::SIGINT => {
                println!("rcvd [SIGINT]");
            },
            libc::SIGTERM => {
                println!("rcvd [SIGTERM], bye.");
                break;
            },
            libc::SIGHUP => {
                println!("rcvd [SIGHUP], reload config")
            },
            _ => {
                eprintln!("unknown signo: {}", si.ssi_signo);
                break;
            }
        }
    }

    Ok(())
}
