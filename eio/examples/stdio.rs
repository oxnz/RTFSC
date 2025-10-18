use std::time::Duration;

use eio::{Action, Event, EventQueue, Filter};

fn main() -> std::io::Result<()> {
    let mut q = EventQueue::try_new()?;
    let mut events = vec![
        Event::new(
            libc::STDIN_FILENO as usize,
            Action::add(),
            Filter::new().read(),
        ),
        Event::new(
            libc::STDOUT_FILENO as usize,
            Action::add(),
            Filter::new().write(),
        ),
    ];
    q.update(&events)?;
    loop {
        match q.poll(&mut events, Some(Duration::from_secs(2))) {
            Ok(n) => {
                eprintln!("nready = {n}");
                for (i, event) in events[..n].iter().enumerate() {
                    eprintln!("event {i}: {event:?}");
                    if event.readable() {
                        let mut buf: Vec<u8> = vec![0; 10];
                        let cnt = unsafe {
                            libc::read(event.ident() as i32, buf.as_mut_ptr().cast(), buf.len())
                        };
                        eprintln!("read {cnt} bytes => [{buf:02x?}]");
                        if cnt == 0 {
                            unsafe {
                                libc::close(event.ident() as i32);
                            }
                        }
                    }
                    if event.writable() {
                        unsafe {
                            let cnt =
                                libc::write(event.ident() as i32, b"hello\n".as_ptr().cast(), 6);
                            eprintln!("write: {cnt}");
                            assert_ne!(-1, libc::close(event.ident() as i32));
                        }
                    }
                    if event.error() {
                        eprintln!("error: {event:?}");
                        unsafe {
                            libc::close(event.ident() as i32);
                        }
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                eprintln!("timeout");
            }
            Err(e) => {
                eprintln!("error: {e:?}");
            }
        }
    }
    Ok(())
}
