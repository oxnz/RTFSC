use std::ffi::CStr;

fn main() {
    let hostname = unsafe {
        let n = libc::sysconf(libc::_SC_HOST_NAME_MAX);
        if n < 0 {
            panic!("sysconf");
        }
        let mut buf: Box<[libc::c_char]> = Box::new_uninit_slice(n as usize).assume_init();
        let r = libc::gethostname(buf.as_mut_ptr(), buf.len());
        assert_eq!(r, 0, "gethostname");
        CStr::from_ptr(buf.as_ptr()).to_owned()
    };
    let hints = {
        let mut p: libc::addrinfo = unsafe { std::mem::zeroed() };
        p.ai_flags = libc::AI_CANONNAME;
        p.ai_socktype = libc::SOCK_DGRAM;
        p.ai_canonname = std::ptr::null_mut();
        p.ai_addr = std::ptr::null_mut();
        p.ai_next = std::ptr::null_mut();
        p
    };
    let mut ailist = std::ptr::null_mut();
    #[cfg(target_os = "macos")]
    let hostname = b"127.0.0.1\0";
    unsafe {
        let r = libc::getaddrinfo(
            hostname.as_ptr() as *const _,
            b"who\0".as_ptr() as *const libc::c_char,
            &hints,
            &mut ailist,
        );
        assert_eq!(r, 0);
    };
    while let Some(addr) = unsafe { ailist.as_ref() } {
        ailist = addr.ai_next;
        let sockfd = unsafe { libc::socket(addr.ai_family, libc::SOCK_STREAM, 0) };
        if sockfd < 0 {
            panic!("socket");
        }
        if unsafe { libc::bind(sockfd, addr.ai_addr, addr.ai_addrlen) } < 0 {
            panic!("bind");
        }
        if unsafe { libc::listen(sockfd, 5) } < 0 {
            panic!("listen");
        }
        serve(sockfd);
    }
}

fn serve(srv_fd: libc::c_int) {
    let mut buf: Box<[i8]> = unsafe { Box::new_uninit_slice(libc::BUFSIZ as usize).assume_init() };
    set_cloexec(srv_fd).unwrap();

    loop {
        let cli_fd = unsafe { libc::accept(srv_fd, std::ptr::null_mut(), std::ptr::null_mut()) };
        if cli_fd < 0 {
            panic!("accept");
        }
        set_cloexec(cli_fd).unwrap();
        let fp = unsafe {
            libc::popen(
                b"/usr/bin/uptime\0".as_ptr() as *const _,
                b"\r\0".as_ptr() as *const _,
            )
        };
        if fp.is_null() {
            unsafe { libc::send(cli_fd, buf.as_ptr() as *const _, 5, 0) };
            panic!("popen");
        } else {
            while !unsafe { libc::fgets(buf.as_mut_ptr(), libc::BUFSIZ as i32, fp).is_null() } {
                unsafe {
                    libc::send(
                        cli_fd,
                        buf.as_ptr() as *const _,
                        libc::strlen(buf.as_ptr()),
                        0,
                    )
                };
            }
            unsafe { libc::pclose(fp) };
        }
        unsafe { libc::close(cli_fd) };
    }
}

fn set_cloexec(fd: libc::c_int) -> Result<(), std::io::Error> {
    let mut val = unsafe { libc::fcntl(fd, libc::F_GETFD, 0) };
    if val < 0 {
        panic!("fcntl");
    }
    val |= libc::FD_CLOEXEC;
    let r = unsafe { libc::fcntl(fd, libc::F_SETFD, val) };
    if r < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}
