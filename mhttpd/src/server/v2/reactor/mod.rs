use std::{
    collections::HashMap,
    net::{SocketAddr, TcpListener, ToSocketAddrs},
    os::fd::AsRawFd,
};

use eio::{Action, Event, EventQueue, Filter};

use crate::http::v2::Connection;

#[derive(Debug, Default)]
pub struct Server {
    connections: HashMap<usize, Connection>,
}

impl Server {
    pub fn serve<A: ToSocketAddrs>(&mut self, addr: A) -> std::io::Result<()> {
        let socket = TcpListener::bind(addr)?;
        let mut event_q = EventQueue::try_new()?;
        let mut events = [Event::new(
            socket.as_raw_fd() as usize,
            Action::add(),
            Filter::new().read(),
        ); 128];
        event_q.register(&events[..1])?;
        loop {
            match event_q.poll(events.as_mut_slice(), None) {
                Ok(n) => {
                    for event in &events[..n] {
                        if event.ident() == socket.as_raw_fd() as usize {
                            if event.readable() {
                                match socket.accept() {
                                    Ok((stream, remote_addr)) => {
                                        tracing::debug!("accept: {remote_addr:?}");
                                        let ident = stream.as_raw_fd() as usize;
                                        let event =
                                            Event::new(ident, Action::add(), Filter::new().read());
                                        let conn = Connection::from(stream);
                                        self.connections.insert(ident, conn);
                                        event_q.register(&[event])?;
                                    }
                                    Err(e) => tracing::error!("accept: {e:?}"),
                                }
                            } else {
                                tracing::error!("unexpected event: {event:?}");
                            }
                        } else {
                            tracing::debug!("event: {event:?}");
                            if event.readable() {}
                        }
                    }
                }
                Err(e) => todo!(),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serve() {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
        let mut server = Server::default();
        let addr = "127.0.0.1:8000";
        server.serve(addr).unwrap();
    }

    #[test]
    fn test_get() {
        let output = std::process::Command::new("curl")
            .args([
                "--http2-prior-knowledge",
                "-v",
                "--silent",
                "127.0.0.1:8000",
            ])
            .output()
            .unwrap();
        let stdout = output.stdout;
        let stderr = output.stderr;
        unsafe {
            println!("stdout:\n{}", str::from_utf8_unchecked(&stdout));
            eprintln!("stderr:\n{}", str::from_utf8_unchecked(&stderr));
        }
    }
}
