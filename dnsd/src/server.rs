use std::net::UdpSocket;

use crate::{SerDe, dns::message::Message};

fn process(request: Message) -> std::io::Result<Message> {
    tracing::debug!("request: {request:?}");
    Ok(request)
}

pub fn serve() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:8000")?;
    let mut buf = [0; 4096];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((n, remote_addr)) => {
                println!("rcvd {n} from {remote_addr:?}");
                let raw_request = &buf[..n];
                println!("raw: [{raw_request:?}]");
                let request = Message::deserialize(raw_request)?;
                let response = process(request)?;
                let mut v = Vec::new();
                response.serialize(&mut v)?;
                socket.send_to(&v, remote_addr).unwrap();
            }
            Err(e) => println!("error: {e:?}"),
        }
    }
    Ok(())
}
