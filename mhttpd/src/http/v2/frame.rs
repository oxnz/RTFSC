use std::io::Write;

use bytes::BufMut;

use crate::http::{
    Header, SerDe,
    codec::{Decode, Encode},
    v2::{ErrorCode, settings::Setting},
};

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
        payload.put_u32(last_stream_id);
        payload.put_u32(error_code as u32);
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
        // let mut header = [0u8; 9];
        //         stream.write_all(&len.to_be_bytes()[1..])?;
        //         stream.write_all(&[Type::RstStream.into()])?; // type
        //         stream.write_all(&[0])?; // flags
        //         stream.write_all(&stream_id.to_be_bytes())?;
        //         stream.write_all(&data.to_be_bytes())?;
        // stream.write_all(self.header.as_slice())?;
        // if let Some(payload) = &self.payload {
        //     stream.write_all(&payload)?;
        // }
        Ok(())
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
        flags: u8,
        data: Vec<u8>,
    },
    Headers {
        stream_id: u32,
        flags: u8,
        items: Vec<Header>,
    },
    RstStream {
        stream_id: u32,
        error_code: ErrorCode,
    },
    Settings {
        ack: u8,
        items: Vec<Setting>,
    },
    WindowUpdate {
        stream_id: u32,
        increment: u32,
    },
}

impl Frame {
    pub fn decode(
        header: &[u8; 9],
        payload: &[u8],
        decoder: &mut hpack::Decoder,
    ) -> std::io::Result<Self> {
        let len =
            (((header[0] as u32) << 16) | ((header[1] as u32) << 8) | (header[2] as u32)) as usize;
        let frame_type: Type = header[3].into();
        let flags = header[4];
        let stream_id = ((header[5] as u32 & 0x7F) << 24)
            | ((header[6] as u32) << 16)
            | ((header[7] as u32) << 8)
            | (header[8] as u32);
        match frame_type {
            Type::Data => {
                let data = if 0 == flags & super::flags::PADDED {
                    payload.to_vec()
                } else {
                    let pad_length = payload[0] as usize;
                    payload[1..len - pad_length].to_vec()
                };
                Ok(Self::Data {
                    stream_id,
                    flags,
                    data,
                })
            }
            Type::Headers => match decoder.decode(&payload) {
                Ok(items) => Ok(Self::Headers {
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
            Type::RstStream => Ok(Self::RstStream {
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
                Ok(Self::Settings { ack: flags, items })
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
                    Ok(Self::WindowUpdate {
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

    pub fn encode<W: Write>(
        &self,
        stream: &mut W,
        encoder: &mut hpack::Encoder,
    ) -> std::io::Result<()> {
        match self {
            Frame::Settings { ack: flags, items } => {
                let len: u32 = items.len() as u32 * 6;
                stream.write_all(&len.to_be_bytes()[1..])?;
                stream.write_all(&[0x04])?; // type
                stream.write_all(&[*flags])?; // flags
                stream.write_all(&0u32.to_be_bytes())?; // stream_id
                for item in items {
                    stream.write_all(&item.identifier().to_be_bytes())?;
                    stream.write_all(&item.value().to_be_bytes())?;
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
            Frame::Headers {
                stream_id,
                flags,
                items,
            } => {
                let payload = encoder.encode(
                    items
                        .iter()
                        .map(|item| (item.name().as_bytes(), item.value().as_bytes())),
                );
                let len: u32 = payload.len() as u32;
                stream.write_all(&len.to_be_bytes()[1..])?;
                stream.write_all(&[0x01])?; // type
                stream.write_all(&[*flags])?; // flags
                stream.write_all(&stream_id.to_be_bytes())?;
                stream.write_all(&payload)?;
            }
            Frame::Data {
                stream_id,
                flags,
                data,
            } => {
                let len: u32 = data.len() as u32;
                stream.write_all(&len.to_be_bytes()[1..])?;
                stream.write_all(&[0x00])?; // type
                stream.write_all(&[*flags])?; // flags
                stream.write_all(&stream_id.to_be_bytes())?;
                stream.write_all(&data)?;
            }
            Frame::RstStream {
                stream_id,
                error_code,
            } => {
                let len = 4u32;
                let data = u32::from(*error_code);
                stream.write_all(&len.to_be_bytes()[1..])?;
                stream.write_all(&[Type::RstStream.into()])?; // type
                stream.write_all(&[0])?; // flags
                stream.write_all(&stream_id.to_be_bytes())?;
                stream.write_all(&data.to_be_bytes())?;
            }
        }
        Ok(())
    }
}

impl SerDe for Frame {
    fn read<R: std::io::BufRead>(stream: &mut R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        // read header
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

        //  Read payload ---
        let mut payload = vec![0u8; len as usize];
        stream.read_exact(&mut payload)?;
        match frame_type {
            Type::Data => {
                let data = if 0 == flags & super::flags::PADDED {
                    payload
                } else {
                    let pad_length = payload[0] as usize;
                    payload[1..len - pad_length].to_vec()
                };
                Ok(Self::Data {
                    stream_id,
                    flags,
                    data,
                })
            }
            Type::Headers => {
                let mut decoder = hpack::Decoder::new();
                let items = decoder.decode(&payload).unwrap();
                Ok(Self::Headers {
                    stream_id,
                    flags,
                    items: vec![],
                })
            }
            Type::RstStream => Ok(Self::RstStream {
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
                Ok(Self::Settings { ack: flags, items })
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
                    Ok(Self::WindowUpdate {
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

    fn write<W: std::io::Write>(&self, stream: &mut W) -> std::io::Result<()> {
        match self {
            Frame::Settings { ack: flags, items } => {
                let len: u32 = items.len() as u32 * 6;
                stream.write_all(&len.to_be_bytes()[1..])?;
                stream.write_all(&[0x04])?; // type
                stream.write_all(&[*flags])?; // flags
                stream.write_all(&0u32.to_be_bytes())?; // stream_id
                for item in items {
                    item.write(stream)?;
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
            Frame::Headers {
                stream_id,
                flags,
                items,
            } => {
                let mut encoder = hpack::Encoder::new();
                let payload = vec![]; // encoder.encode(items.iter().map(|item| (&item.0[..], &item.1[..])));
                let len: u32 = payload.len() as u32;
                stream.write_all(&len.to_be_bytes()[1..])?;
                stream.write_all(&[0x01])?; // type
                stream.write_all(&[*flags])?; // flags
                stream.write_all(&stream_id.to_be_bytes())?;
                stream.write_all(&payload)?;
            }
            Frame::Data {
                stream_id,
                flags,
                data,
            } => {
                let len: u32 = data.len() as u32;
                stream.write_all(&len.to_be_bytes()[1..])?;
                stream.write_all(&[0x00])?; // type
                stream.write_all(&[*flags])?; // flags
                stream.write_all(&stream_id.to_be_bytes())?;
                stream.write_all(&data)?;
            }
            Frame::RstStream {
                stream_id,
                error_code,
            } => {
                let len = 4u32;
                let data = u32::from(*error_code);
                stream.write_all(&len.to_be_bytes()[1..])?;
                stream.write_all(&[Type::RstStream.into()])?; // type
                stream.write_all(&[0])?; // flags
                stream.write_all(&stream_id.to_be_bytes())?;
                stream.write_all(&data.to_be_bytes())?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_data() {
        let raw = [0, 0, 0, 0, 0, 0, 0, 0, 0];
        let frame = Frame::read(&mut std::io::Cursor::new(raw)).unwrap();
        println!("{frame:?}");
        let mut v = Vec::new();
        frame.write(&mut v).unwrap();
        println!("{v:?}");
        assert_eq!(raw, v.as_slice());
    }

    #[test]
    fn test_serde_headers() {
        let raw = [
            0, 0, 30, 1, 5, 0, 0, 0, 1, 130, 134, 65, 138, 8, 157, 92, 11, 129, 112, 220, 120, 0,
            7, 132, 122, 136, 37, 182, 80, 195, 203, 186, 184, 127, 83, 3, 42, 47, 42,
        ];
        let frame = Frame::read(&mut std::io::Cursor::new(raw)).unwrap();
        println!("{frame:?}");
        let mut v = Vec::new();
        frame.write(&mut v).unwrap();
        println!("{v:?}");
        // assert_eq!(raw, v.as_slice());
    }

    #[test]
    fn test_serde_settings() {
        let raw = [
            0, 0, 18, 4, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 100, 0, 4, 0, 160, 0, 0, 0, 2, 0, 0, 0, 0,
        ];
        let frame = Frame::read(&mut std::io::Cursor::new(raw)).unwrap();
        println!("{frame:?}");
        let mut v = Vec::new();
        frame.write(&mut v).unwrap();
        println!("{v:?}");
        assert_eq!(raw, v.as_slice());
    }

    #[test]
    fn test_serde_window_update() {
        let raw = [0, 0, 4, 8, 0, 0, 0, 0, 0, 62, 127, 0, 1];
        let frame = Frame::read(&mut std::io::Cursor::new(raw)).unwrap();
        println!("{frame:?}");
        let mut v = Vec::new();
        frame.write(&mut v).unwrap();
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
