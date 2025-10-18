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
        Event::new(
            libc::STDERR_FILENO as usize,
            Action::add(),
            Filter::new().write(),
        ),
    ];
    q.update(&events)?;
    loop {
        println!("loop");
        match q.poll(&mut events, Some(Duration::from_millis(100))) {
            Ok(n) => {
                println!("{n} events ready");
                for event in &events[..n] {
                    println!("event: {event:?}");
                    if event.readable() {
                        println!("read: {event:?}");
                        let mut buf: Vec<u8> = vec![0; 10];
                        let cnt = unsafe {
                            libc::read(event.ident() as i32, buf.as_mut_ptr().cast(), buf.len())
                        };
                        println!("read {cnt} bytes => [{buf:02x?}]");
                    }
                    if event.writable() {
                        println!("write: {event:?}");
                        unsafe {
                            println!("begin write");
                            libc::write(event.ident() as i32, b"hello\n".as_ptr().cast(), 6);
                            println!("end write");
                            assert_ne!(-1, libc::close(event.ident() as i32));
                        }
                    }
                    if event.error() {
                        println!("error: {event:?}");
                        unsafe {
                            libc::close(event.ident() as i32);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("error: {e:?}");
            }
        }
    }
    Ok(())
}
