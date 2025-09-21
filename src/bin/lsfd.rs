use std::process::ExitCode;

fn main() -> Result<(), ExitCode> {
    let argv = std::env::args().collect::<Vec<_>>();
    if argv.len() != 2 {
        eprintln!("usage: {} <descriptor>", argv[0]);
        return Err(ExitCode::FAILURE);
    }
    let fd = argv[1].parse::<i32>().expect("invalid fd");
    let value = unsafe { libc::fcntl(fd, libc::F_GETFL, 0) };
    if value < 0 {
        unsafe { libc::perror("fcntl".as_ptr() as *const _) };
        return Err(ExitCode::FAILURE);
    }

    let access_mode = match value & libc::O_ACCMODE {
        libc::O_RDONLY => "read only",
        libc::O_WRONLY => "write only",
        libc::O_RDWR => "read & write",
        _ => "uknown",
    };

    println!("access mode: {access_mode}");

    if 0 != value & libc::O_APPEND {
        println!("append");
    }
    if 0 != value & libc::O_NONBLOCK {
        println!("nonblocking");
    }
    if 0 != value & libc::O_SYNC {
        println!("sync write");
    }
    if libc::O_FSYNC != libc::O_SYNC && 0 != value & libc::O_FSYNC {
        println!("sync write");
    }

    Ok(())
}
