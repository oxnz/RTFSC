use std::collections::HashMap;

use tokio::net::TcpStream;

#[derive(Debug)]
pub struct Transport {
    stream: TcpStream,
    streams: HashMap<u32, Vec<u8>>,
}
