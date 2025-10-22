pub mod flags;
mod frame;
mod header;
mod settings;
pub use frame::Frame;

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
