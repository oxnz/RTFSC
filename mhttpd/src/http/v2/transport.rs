use std::collections::HashMap;

use crate::http::v2::Connection;

#[derive(Debug)]
pub struct Transport {
    connection: Connection,
    streams: HashMap<u32, Stream>,
}

#[derive(Debug)]
pub struct Stream {}
