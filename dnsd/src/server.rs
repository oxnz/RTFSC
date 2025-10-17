use std::net::UdpSocket;

use crate::{SerDe, dns::message::Message};

pub fn serve() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:8000")?;
    let mut buf = [0; 4096];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((n, remote_addr)) => {
                println!("rcvd {n} from {remote_addr:?}");
                let request = &buf[..n];
                println!("raw: [{request:?}]");
                let query = Message::deserialize(request);
                println!("{query:?}");
                let response = request;
                socket.send_to(response, remote_addr).unwrap();
            }
            Err(e) => println!("error: {e:?}"),
        }
    }
    Ok(())
}
