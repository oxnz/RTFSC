use std::{ffi::CString, mem::MaybeUninit, str::FromStr};

use rtfsc::{file_op::set_fl, flock::Flock, signal::SigSet};

fn main() {
    let argv = std::env::args().collect::<Vec<_>>();
    if argv.len() != 2 {
        eprintln!("usage: {} filename", argv[0]);
        return;
    }
    let path = CString::from_str(&argv[1]).unwrap();
    let fd = unsafe {
        libc::open(
            path.as_ptr(),
            libc::O_RDWR | libc::O_CREAT | libc::O_TRUNC,
            (libc::S_IRUSR | libc::S_IWUSR | libc::S_IRGRP | libc::S_IROTH) as libc::c_int,
        )
    };
    unsafe {
        let buf = b"abcdef";
        assert_eq!(
            buf.len() as isize,
            libc::write(fd, buf.as_ptr().cast(), buf.len())
        );

        /*
        turn on set-group-ID and turn off group-execute
        Mandatory locking is enabled for a particular file by turning on the set-group-ID bit
        and turning off the group-execute bit. (Recall Figure 4.12.) Since the set-group-ID bit
        makes no sense when the group-execute bit is off, the designers of SVR3 chose this way
        to specify that the locking for a file is to be mandatory locking and not advisory locking.
        */
        let stat = {
            let mut p = MaybeUninit::uninit();
            libc::fstat(fd, p.as_mut_ptr());
            p.assume_init()
        };
        libc::fchmod(fd, (stat.st_mode & !libc::S_IXGRP) | libc::S_ISGID);
    }
    let mask = SigSet::from([libc::SIGUSR1, libc::SIGUSR2].as_slice());
    mask.block();

    let pid = unsafe { libc::fork() };
    if pid < 0 {
        panic!("fork");
    } else if pid > 0 {
        // parent
        // write lock entire file
        Flock::write(std::io::SeekFrom::Start(0), 0)
            .set(fd)
            .unwrap();
        println!("parent wake child");
        unsafe { libc::kill(pid, libc::SIGUSR2) };
        println!("parent sent [{}]", libc::SIGUSR2);
        unsafe {
            libc::waitpid(pid, std::ptr::null_mut(), 0);
        }
    } else if pid == 0 {
        // child
        println!("child wait for parent");
        let signo = mask.wait();
        println!("child rcvd [{signo}]");
        set_fl(fd, libc::O_NONBLOCK);

        // first let’s see what error we get if region is locked
        // no wait
        match Flock::read(std::io::SeekFrom::Start(0), 0).set(fd) {
            Ok(_) => println!("child: read lock set succeeded"),
            Err(e) => eprintln!("child: failed to set read lock, error: {e:?}"),
        }

        // now try to read the mandatory locked file
        if -1 == unsafe { libc::lseek(fd, 0, libc::SEEK_SET) } {
            eprintln!("lseek failed");
        }
        let mut buf: [u8; 2] = [0; 2];
        if unsafe { libc::read(fd, buf.as_mut_ptr().cast(), buf.len()) } < 0 {
            eprintln!("read failed, error: {:?}", std::io::Error::last_os_error());
        } else {
            println!("read succeeded (no mandatory locking), buf: {:?}", buf);
        }
    }
}
