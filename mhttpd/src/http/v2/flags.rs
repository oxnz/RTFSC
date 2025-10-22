pub const END_STREAM: u8 = 0x01;
pub const ACK: u8 = 0x01; // for PING and SETTINGS
pub const END_HEADERS: u8 = 0x04; // for HEADERS, CONTINUATION, PUSH_PROMISE
pub const PADDED: u8 = 0x08; // DATA, HEADERS, PUSH_PROMISE
pub const PRIORITY: u8 = 0x20; // HEADERS only
