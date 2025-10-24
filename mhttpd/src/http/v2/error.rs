#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum Error {
    NoError = 0x00,
    ProtocolError = 0x01,
    InternalError = 0x02,
    FlowControlError = 0x03,
    SettingsTimeout = 0x04,
    StreamClosed = 0x05,
    FrameSizeError = 0x06,
    RefusedStream = 0x07,
    Cancel = 0x08,
    CompressionError = 0x09,
    ConnectError = 0x0a,
    EnhanceYourCalm = 0x0b,
    InadequateSecurity = 0x0c,
    Http11Required = 0x0d,
}

impl From<u32> for Error {
    fn from(value: u32) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl From<Error> for u32 {
    fn from(value: Error) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conv() {
        assert!(matches!(1u32.into(), Error::ProtocolError));
        assert_eq!(u32::from(Error::ProtocolError), 1u32);
    }
}
