use crate::http::SerDe;

mod settings;

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

impl From<u8> for FrameType {
    fn from(value: u8) -> Self {
        unsafe { std::mem::transmute::<u8, Self>(value) }
    }
}

#[derive(Debug)]
pub struct Frame {
    r#type: FrameType,
    flags: u8,
    stream_id: u32,
    payload: Vec<u8>,
}

impl Frame {
    pub fn new(r#type: FrameType, flags: u8, stream_id: u32, payload: Vec<u8>) -> Self {
        Self {
            r#type,
            flags,
            stream_id,
            payload,
        }
    }
}

impl SerDe for Frame {
    fn read<R: std::io::BufRead>(stream: &mut R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        // read header
        let mut header = [0u8; 9];
        let n = stream.read(&mut header)?;
        let len = ((header[0] as u32) << 16) | ((header[1] as u32) << 8) | (header[2] as u32);
        let frame_type = header[3];
        let flags = header[4];
        let stream_id = ((header[5] as u32 & 0x7F) << 24)
            | ((header[6] as u32) << 16)
            | ((header[7] as u32) << 8)
            | (header[8] as u32);

        //  Read payload ---
        let mut payload = vec![0u8; len as usize];
        stream.read_exact(&mut payload)?;
        match frame_type {
            0x4 => {
                // SETTINGS
                tracing::info!("Received SETTINGS frame");
                // Normally you parse key-value settings pairs here
            }
            0x1 => {
                // HEADERS
                tracing::info!("Received HEADERS frame ({} bytes)", payload.len());
                tracing::info!(
                    "Raw header payload: {:x?}",
                    &payload[..payload.len().min(32)]
                );
            }
            _ => tracing::info!("Other frame type 0x{:x}", frame_type),
        }
        Ok(Self {
            r#type: frame_type.into(),
            flags,
            stream_id,
            payload,
        })
    }

    fn write<W: std::io::Write>(&self, stream: &mut W) -> std::io::Result<()> {
        let headers_len = self.payload.len();
        let mut headers_frame = Vec::with_capacity(9 + headers_len);
        headers_frame.push(((headers_len >> 16) & 0xFF) as u8);
        headers_frame.push(((headers_len >> 8) & 0xFF) as u8);
        headers_frame.push((headers_len & 0xFF) as u8);
        headers_frame.push(self.r#type.clone() as u8); // HEADERS
        headers_frame.push(self.flags); // END_HEADERS | END_STREAM
        headers_frame.push(self.stream_id as u8); // stream id MSB
        headers_frame.push(0);
        headers_frame.push(0);
        headers_frame.push(1); // stream id LSB = 1
        stream.write_all(&headers_frame)?;
        stream.write_all(&self.payload)?;
        Ok(())
    }
}
