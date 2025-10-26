use crate::http::{
    Header, SerDe,
    codec::{Codec, Decode, Encode},
    v2::{ErrorCode, flags, settings::Setting},
};
use std::{io::Write, num::NonZero};

pub(crate) struct RawFrame {
    r#type: Type,
    flags: u8,
    stream_id: u32,
    payload: Option<Vec<u8>>,
}

impl RawFrame {
    pub fn reset_stream(stream_id: u32) -> Self {
        Self {
            r#type: Type::RstStream,
            flags: 0,
            stream_id,
            payload: None,
        }
    }

    pub fn ping(ack: bool, opaque_data: Vec<u8>) -> Self {
        Self {
            r#type: Type::Ping,
            flags: if ack { 1 } else { 0 },
            stream_id: 0,
            payload: Some(opaque_data),
        }
    }

    pub fn goaway(
        last_stream_id: u32,
        error_code: ErrorCode,
        additional_debug_data: Option<&[u8]>,
    ) -> Self {
        let mut payload = Vec::with_capacity(
            8 + additional_debug_data
                .map(|data| data.len())
                .unwrap_or_default(),
        );
        payload.write_all(&last_stream_id.to_be_bytes()).unwrap();
        payload
            .write_all(&(error_code as u32).to_be_bytes())
            .unwrap();
        if let Some(data) = additional_debug_data {
            payload.extend_from_slice(data);
        }
        Self {
            r#type: Type::GoAway,
            flags: 0,
            stream_id: last_stream_id,
            payload: Some(payload),
        }
    }

    pub fn window_update(stream_id: u32, increment: u32) -> Self {
        Self {
            r#type: Type::WindowUpdate,
            flags: 0,
            stream_id,
            payload: Some(increment.to_be_bytes().to_vec()),
        }
    }
}

impl SerDe for RawFrame {
    fn read<R: std::io::BufRead>(stream: &mut R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let mut header = [0u8; 9];
        stream.read_exact(&mut header)?;
        let len =
            (((header[0] as u32) << 16) | ((header[1] as u32) << 8) | (header[2] as u32)) as usize;
        let frame_type: Type = header[3].into();
        let flags = header[4];
        let stream_id = ((header[5] as u32 & 0x7F) << 24)
            | ((header[6] as u32) << 16)
            | ((header[7] as u32) << 8)
            | (header[8] as u32);
        Ok(if len > 0 {
            let mut payload = vec![0u8; len];
            stream.read_exact(&mut payload)?;
            Self {
                r#type: frame_type,
                flags,
                stream_id,
                payload: Some(payload),
            }
        } else {
            Self {
                r#type: frame_type,
                flags,
                stream_id,
                payload: None,
            }
        })
    }

    fn write<W: Write>(&self, stream: &mut W) -> std::io::Result<()> {
        let len = self.payload.as_ref().map(|x| x.len()).unwrap_or_default();
        let mut buffer = Vec::with_capacity(9 + len);
        buffer.write_all(&(len as u32).to_be_bytes()[1..])?;
        buffer.write_all(&[Type::RstStream.into()])?; // type
        buffer.write_all(&[0])?; // flags
        buffer.write_all(&self.stream_id.to_be_bytes())?;
        if let Some(data) = &self.payload {
            buffer.write_all(data)?;
        }
        stream.write_all(&buffer)
    }
}

#[derive(Debug)]
pub enum Type {
    Data,
    Headers,
    Priority,
    RstStream,
    Settings,
    PushPromise,
    Ping,
    GoAway,
    WindowUpdate,
    Continuation,
    Unknown(u8),
}

impl From<u8> for Type {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::Data,
            0x01 => Self::Headers,
            0x02 => Self::Priority,
            0x03 => Self::RstStream,
            0x04 => Self::Settings,
            0x05 => Self::PushPromise,
            0x06 => Self::Ping,
            0x07 => Self::GoAway,
            0x08 => Self::WindowUpdate,
            0x09 => Self::Continuation,
            _ => Self::Unknown(value),
        }
    }
}

impl From<Type> for u8 {
    fn from(value: Type) -> Self {
        match value {
            Type::Data => 0x00,
            Type::Headers => 0x01,
            Type::Priority => 0x02,
            Type::RstStream => 0x03,
            Type::Settings => 0x04,
            Type::PushPromise => 0x05,
            Type::Ping => 0x06,
            Type::GoAway => 0x07,
            Type::WindowUpdate => 0x08,
            Type::Continuation => 0x09,
            Type::Unknown(_value) => _value,
        }
    }
}

#[derive(Debug)]
pub enum Frame {
    Data {
        stream_id: u32,
        end_stream: bool,
        pad_len: Option<u8>,
        data: Vec<u8>,
    },
    Headers {
        stream_id: u32,
        flags: u8,
        items: Vec<Header>,
    },
    Priority {
        stream_id: u32,
        exclusive: bool,
        stream_dependency: u32,
        weight: u8,
    },
    RstStream {
        stream_id: u32,
        error_code: ErrorCode,
    },
    Settings {
        ack: u8,
        items: Vec<Setting>,
    },
    PushPromise {
        stream_id: u32,
        end_headers: bool,
        headers: Vec<Header>,
        pad_len: Option<u8>,
        promised_stream_id: u32,
    },
    Ping {
        ack: bool,
        opaque_data: u64,
    },
    GoAway {
        last_stream_id: u32,
        error_code: ErrorCode,
        debug_data: Option<Vec<u8>>,
    },
    WindowUpdate {
        stream_id: u32,
        increment: u32, // non-zero
    },
    Continuation {
        end_headers: bool,
        stream_id: u32,
        headers: Vec<Header>,
    },
}

pub struct FrameCodec {
    hpack_encoder: hpack::Encoder<'static>,
    hpack_decoder: hpack::Decoder<'static>,
}

impl Default for FrameCodec {
    fn default() -> Self {
        Self {
            hpack_encoder: hpack::Encoder::new(),
            hpack_decoder: hpack::Decoder::new(),
        }
    }
}

impl Decode<Frame> for FrameCodec {
    fn decode<R: std::io::BufRead>(&mut self, stream: &mut R) -> std::io::Result<Frame> {
        let mut header = [0u8; 9];
        stream.read_exact(&mut header)?;
        let len =
            (((header[0] as u32) << 16) | ((header[1] as u32) << 8) | (header[2] as u32)) as usize;
        let frame_type: Type = header[3].into();
        let flags = header[4];
        let stream_id = ((header[5] as u32 & 0x7F) << 24)
            | ((header[6] as u32) << 16)
            | ((header[7] as u32) << 8)
            | (header[8] as u32);
        let mut payload = vec![0u8; len];
        stream.read_exact(&mut payload)?;
        match frame_type {
            Type::Data => {
                let end_stream = 0 != flags & flags::END_STREAM;
                let pad_len = if 0 == flags & flags::PADDED {
                    None
                } else {
                    Some(payload[0])
                };
                let data = pad_len
                    .map(|n| payload[1..len - n as usize].to_vec())
                    .unwrap_or(payload);
                Ok(Frame::Data {
                    stream_id,
                    end_stream,
                    pad_len,
                    data,
                })
            }
            Type::Headers => match self.hpack_decoder.decode(&payload) {
                Ok(items) => Ok(Frame::Headers {
                    stream_id,
                    flags,
                    items: items
                        .into_iter()
                        .map(|(k, v)| {
                            Header::new(str::from_utf8(&k).unwrap(), str::from_utf8(&v).unwrap())
                        })
                        .collect(),
                }),
                Err(e) => {
                    tracing::error!("header decode failed: {e:?}, payload: {payload:?}");
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "invalid header",
                    ));
                }
            },
            Type::RstStream => Ok(Frame::RstStream {
                stream_id,
                error_code: ErrorCode::from(u32::from_be_bytes(
                    *payload.first_chunk::<4>().unwrap(),
                )),
            }),
            Type::Settings => {
                let mut items = Vec::new();
                let n = payload.len() / 6;
                let mut stream = std::io::Cursor::new(payload);
                for _i in 0..n {
                    let setting = Setting::read(&mut stream)?;
                    items.push(setting);
                }
                Ok(Frame::Settings { ack: flags, items })
            }
            Type::WindowUpdate => {
                if payload.len() == 4 {
                    let raw = u32::from_be_bytes(payload.try_into().unwrap());
                    let increment = raw & 0x7FFF_FFFF; // mask off reserved bit
                    if increment == 0 {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "non-zero value expected",
                        ));
                    }
                    Ok(Frame::WindowUpdate {
                        stream_id,
                        increment,
                    })
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "4 bytes expected",
                    ))
                }
            }
            _ => {
                tracing::info!("Other frame type {:?}", frame_type);
                panic!();
            }
        }
    }
}

impl Encode<Frame> for FrameCodec {
    fn encode<W: std::io::Write>(&mut self, item: Frame, stream: &mut W) -> std::io::Result<()> {
        match item {
            Frame::Data {
                stream_id,
                end_stream,
                pad_len,
                data,
            } => {
                let len: u32 = data.len() as u32 + pad_len.unwrap_or_default() as u32;
                let mut flags = 0;
                if end_stream {
                    flags |= flags::END_STREAM;
                }
                if pad_len.is_some() {
                    flags |= flags::PADDED;
                }
                stream.write_all(&len.to_be_bytes()[1..])?;
                stream.write_all(&[0x00])?; // type
                stream.write_all(&[flags])?; // flags
                stream.write_all(&stream_id.to_be_bytes())?;
                if let Some(n) = pad_len {
                    stream.write_all(&[n])?;
                }
                stream.write_all(&data)?;
                if let Some(n) = pad_len {
                    stream.write_all(&vec![0u8; n as usize])?;
                }
            }
            Frame::Headers {
                stream_id,
                flags,
                items,
            } => {
                let payload = self.hpack_encoder.encode(
                    items
                        .iter()
                        .map(|item| (item.name().as_bytes(), item.value().as_bytes())),
                );
                let len: u32 = payload.len() as u32;
                stream.write_all(&len.to_be_bytes()[1..])?;
                stream.write_all(&[0x01])?; // type
                stream.write_all(&[flags])?; // flags
                stream.write_all(&stream_id.to_be_bytes())?;
                stream.write_all(&payload)?;
            }
            Frame::Priority {
                stream_id,
                exclusive,
                mut stream_dependency,
                weight,
            } => {
                let len = 0x05_u32;
                stream.write_all(&len.to_be_bytes()[1..])?;
                stream.write_all(&[0x02, 0x00])?; // type & flags
                stream.write_all(&stream_id.to_be_bytes())?;
                if exclusive {
                    stream_dependency |= 0x8000_0000;
                }
                stream.write_all(&stream_dependency.to_be_bytes())?;
                stream.write_all(&[weight])?;
            }
            Frame::RstStream {
                stream_id,
                error_code,
            } => {
                let len = 4u32;
                let data = u32::from(error_code);
                stream.write_all(&len.to_be_bytes()[1..])?;
                stream.write_all(&[Type::RstStream.into()])?; // type
                stream.write_all(&[0])?; // flags
                stream.write_all(&stream_id.to_be_bytes())?;
                stream.write_all(&data.to_be_bytes())?;
            }
            Frame::Settings { ack: flags, items } => {
                let len: u32 = items.len() as u32 * 6;
                stream.write_all(&len.to_be_bytes()[1..])?;
                stream.write_all(&[0x04])?; // type
                stream.write_all(&[flags])?; // flags
                stream.write_all(&0u32.to_be_bytes())?; // stream_id
                for item in items {
                    stream.write_all(&item.identifier().to_be_bytes())?;
                    stream.write_all(&item.value().to_be_bytes())?;
                }
            }
            Frame::PushPromise {
                stream_id,
                end_headers,
                headers,
                pad_len,
                promised_stream_id,
            } => {
                let payload = self.hpack_encoder.encode(
                    headers
                        .iter()
                        .map(|item| (item.name().as_bytes(), item.value().as_bytes())),
                );
                let pad_len = pad_len.unwrap_or_default();
                let len = 4 + payload.len() as u32 + pad_len as u32;
                let flags = (if end_headers { flags::END_HEADERS } else { 0 })
                    | if pad_len != 0 { flags::PADDED } else { 0 };
                stream.write_all(&len.to_be_bytes()[1..])?;
                stream.write_all(&[0x09, flags])?;
                stream.write_all(&stream_id.to_be_bytes())?;
                if 0 != pad_len {
                    stream.write_all(&[pad_len])?;
                }
                stream.write_all(&promised_stream_id.to_be_bytes())?;
                stream.write_all(&payload)?;
                if 0 != pad_len {
                    stream.write_all(&vec![0u8; pad_len as usize])?;
                }
            }
            Frame::Ping { ack, opaque_data } => {
                let len = 0x08_u32;
                let flags = if ack { flags::ACK } else { 0x00 };
                stream.write_all(&len.to_be_bytes()[1..])?;
                stream.write_all(&[0x06, flags])?;
                stream.write_all(&0u32.to_be_bytes())?;
                stream.write_all(&opaque_data.to_be_bytes())?;
            }
            Frame::GoAway {
                last_stream_id,
                error_code,
                debug_data,
            } => {
                let len = 8 + debug_data.as_ref().map(|x| x.len()).unwrap_or_default() as u32;
                stream.write_all(&len.to_be_bytes()[1..])?;
                stream.write_all(&[0x07, 0x00, 0x00, 0x00, 0x00, 0x00])?;
                stream.write_all(&last_stream_id.to_be_bytes())?;
                stream.write_all(&(error_code as u32).to_be_bytes())?;
                if let Some(data) = debug_data {
                    stream.write_all(&data)?;
                }
            }
            Frame::WindowUpdate {
                stream_id,
                increment,
            } => {
                let len: u32 = 4;
                stream.write_all(&len.to_be_bytes()[1..])?;
                stream.write_all(&[0x08])?; // type
                stream.write_all(&[0])?; // flags
                stream.write_all(&stream_id.to_be_bytes())?;
                stream.write_all(&increment.to_be_bytes())?;
            }
            Frame::Continuation {
                end_headers,
                stream_id,
                headers,
            } => {
                let payload = self.hpack_encoder.encode(
                    headers
                        .iter()
                        .map(|item| (item.name().as_bytes(), item.value().as_bytes())),
                );
                let len = payload.len() as u32;
                let flags = if end_headers { flags::END_HEADERS } else { 0 };
                stream.write_all(&len.to_be_bytes()[1..])?;
                stream.write_all(&[0x09, flags])?;
                stream.write_all(&stream_id.to_be_bytes())?;
                stream.write_all(&payload)?;
            }
        }
        Ok(())
    }
}

impl Codec<Frame> for FrameCodec {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_data() {
        let raw = [0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut codec = FrameCodec::default();
        let frame = codec.decode(&mut std::io::Cursor::new(raw)).unwrap();
        println!("{frame:?}");
        let mut v = Vec::new();
        codec.encode(frame, &mut v).unwrap();
        println!("{v:?}");
        assert_eq!(raw, v.as_slice());
    }

    #[test]
    fn test_serde_headers() {
        let raw = [
            0, 0, 30, 1, 5, 0, 0, 0, 1, 130, 134, 65, 138, 8, 157, 92, 11, 129, 112, 220, 120, 0,
            7, 132, 122, 136, 37, 182, 80, 195, 203, 186, 184, 127, 83, 3, 42, 47, 42,
        ];
        let mut codec = FrameCodec::default();
        let frame = codec.decode(&mut std::io::Cursor::new(raw)).unwrap();
        println!("{frame:?}");
        let mut v = Vec::new();
        codec.encode(frame, &mut v).unwrap();
        println!("{v:?}");
        // assert_eq!(raw, v.as_slice());
    }

    #[test]
    fn test_serde_settings() {
        let raw = [
            0, 0, 18, 4, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 100, 0, 4, 0, 160, 0, 0, 0, 2, 0, 0, 0, 0,
        ];
        let mut codec = FrameCodec::default();

        let frame = codec.decode(&mut std::io::Cursor::new(raw)).unwrap();
        println!("{frame:?}");
        let mut v = Vec::new();
        codec.encode(frame, &mut v).unwrap();
        println!("{v:?}");
        assert_eq!(raw, v.as_slice());
    }

    #[test]
    fn test_serde_window_update() {
        let raw = [0, 0, 4, 8, 0, 0, 0, 0, 0, 62, 127, 0, 1];
        let mut codec = FrameCodec::default();

        let frame = codec.decode(&mut std::io::Cursor::new(raw)).unwrap();
        println!("{frame:?}");
        let mut v = Vec::new();
        codec.encode(frame, &mut v).unwrap();
        println!("{v:?}");
        assert_eq!(raw, v.as_slice());
    }

    #[test]
    fn test_hpack() {
        let raw = [
            130, 134, 65, 138, 8, 157, 92, 11, 129, 112, 220, 120, 0, 7, 132, 122, 136, 37, 182,
            80, 195, 203, 186, 184, 127, 83, 3, 42, 47, 42,
        ];
        let mut decoder = hpack::Decoder::new();
        let items = decoder.decode(&raw).unwrap();
        let mut encoder = hpack::Encoder::new();
        let result = encoder.encode(items.iter().map(|(k, v)| (&k[..], &v[..])));
        // assert_eq!(raw, result.as_slice());
    }
}
