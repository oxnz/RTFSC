use std::{ffi::c_void, mem::MaybeUninit, process::ExitCode};

#[cfg(target_os = "linux")]
fn main() -> Result<(), ExitCode> {
    let sfd = unsafe {
        let mut mask: MaybeUninit<libc::sigset_t> = MaybeUninit::uninit();
        libc::sigemptyset(mask.as_mut_ptr());
        libc::sigaddset(mask.as_mut_ptr(), libc::SIGINT);
        libc::sigaddset(mask.as_mut_ptr(), libc::SIGTERM);
        libc::sigaddset(mask.as_mut_ptr(), libc::SIGHUP);
        libc::sigprocmask(libc::SIG_BLOCK, mask.as_ptr(), std::ptr::null_mut());
        libc::signalfd(-1, mask.as_ptr(), 0)
    };
    assert!(sfd > 0);

    let tfd = unsafe {
        let tfd = libc::timerfd_create(libc::CLOCK_MONOTONIC, libc::TFD_NONBLOCK);
        assert!(tfd > 0);
        let ts = libc::itimerspec {
            it_interval: libc::timespec {
                tv_sec: 10,
                tv_nsec: 0,
            },
            it_value: libc::timespec {
                tv_sec: 10,
                tv_nsec: 0,
            },
        };
        let r = libc::timerfd_settime(tfd, 0, std::ptr::addr_of!(ts), std::ptr::null_mut());
        assert_ne!(-1, r);
        tfd
    };

    let epfd = unsafe {
        let epfd = libc::epoll_create1(0);
        assert_ne!(-1, epfd);
        let mut ev = libc::epoll_event {
            events: libc::EPOLLIN as u32,
            u64: sfd as u64,
        };
        libc::epoll_ctl(epfd, libc::EPOLL_CTL_ADD, sfd, std::ptr::addr_of_mut!(ev));
        ev.u64 = tfd as u64;
        libc::epoll_ctl(epfd, libc::EPOLL_CTL_ADD, tfd, std::ptr::addr_of_mut!(ev));
        epfd
    };

    let mut events: [libc::epoll_event; 10] = unsafe { MaybeUninit::uninit().assume_init() };
    loop {
        let nfds = unsafe { libc::epoll_wait(epfd, events.as_mut_ptr(), 10, 1000) };
        assert!(nfds >= 0);
        println!("nfds = {nfds}");
        for e in &events[..nfds as usize] {
            if e.u64 == sfd as u64 {
                println!("signal fired");
                let si = {
                    let mut si: MaybeUninit<libc::signalfd_siginfo> = MaybeUninit::uninit();
                    unsafe {
                        let n = libc::read(
                            sfd,
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
                    }
                    libc::SIGTERM => {
                        println!("rcvd [SIGTERM], bye.");
                        break;
                    }
                    libc::SIGHUP => {
                        println!("rcvd [SIGHUP], reload config")
                    }
                    _ => {
                        eprintln!("unknown signo: {}", si.ssi_signo);
                        break;
                    }
                }
            } else if e.u64 == tfd as u64 {
                println!("timer fired");
            } else {
                println!("unknow");
            }
        }
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() {
    panic!("unsupported");
}
