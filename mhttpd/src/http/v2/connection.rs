use std::net::TcpStream;

#[derive(Debug)]
pub struct Connection {
    stream: TcpStream,
}

impl From<TcpStream> for Connection {
    fn from(value: TcpStream) -> Self {
        Self { stream: value }
    }
}
