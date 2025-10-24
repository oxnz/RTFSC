mod connection;
pub mod flags;
pub use connection::Connection;
mod frame;
mod settings;
mod transport;
pub use frame::Frame;
mod preface;
pub use preface::{PREFACE, Preface};
mod error;
pub mod stream;
pub use error::Error as ErrorCode;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum FrameType {
    Data = 0x00,
    Headers = 0x01,
    Priority = 0x02,
    RstStream = 0x03,
    Settings = 0x04,
    PushPromise = 0x05,
    Ping = 0x06,
    GoAway = 0x07,
    WindowUpdate = 0x08,
    Continuation = 0x09,
}
