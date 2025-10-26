use crate::http::{
    Header, SerDe,
    codec::{Codec, Decode, Encode},
    v2::{ErrorCode, Frame, FrameType, settings::Setting},
};

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
        let frame_type: FrameType = header[3].into();
        let flags = header[4];
        let stream_id = ((header[5] as u32 & 0x7F) << 24)
            | ((header[6] as u32) << 16)
            | ((header[7] as u32) << 8)
            | (header[8] as u32);
        let mut payload = vec![0u8; len];
        stream.read_exact(&mut payload)?;
        match frame_type {
            FrameType::Data => {
                let data = if 0 == flags & super::flags::PADDED {
                    payload.to_vec()
                } else {
                    let pad_length = payload[0] as usize;
                    payload[1..len - pad_length].to_vec()
                };
                Ok(Frame::Data {
                    stream_id,
                    flags,
                    data,
                })
            }
            FrameType::Headers => match self.hpack_decoder.decode(&payload) {
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
            FrameType::RstStream => Ok(Frame::RstStream {
                stream_id,
                error_code: ErrorCode::from(u32::from_be_bytes(
                    *payload.first_chunk::<4>().unwrap(),
                )),
            }),
            FrameType::Settings => {
                let mut items = Vec::new();
                let n = payload.len() / 6;
                let mut stream = std::io::Cursor::new(payload);
                for _i in 0..n {
                    let setting = Setting::read(&mut stream)?;
                    items.push(setting);
                }
                Ok(Frame::Settings { ack: flags, items })
            }
            FrameType::WindowUpdate => {
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
            Frame::Data {
                stream_id,
                flags,
                data,
            } => {
                let len: u32 = data.len() as u32;
                stream.write_all(&len.to_be_bytes()[1..])?;
                stream.write_all(&[0x00])?; // type
                stream.write_all(&[flags])?; // flags
                stream.write_all(&stream_id.to_be_bytes())?;
                stream.write_all(&data)?;
            }
            Frame::RstStream {
                stream_id,
                error_code,
            } => {
                let len = 4u32;
                let data = u32::from(error_code);
                stream.write_all(&len.to_be_bytes()[1..])?;
                stream.write_all(&[FrameType::RstStream.into()])?; // type
                stream.write_all(&[0])?; // flags
                stream.write_all(&stream_id.to_be_bytes())?;
                stream.write_all(&data.to_be_bytes())?;
            }
        }
        Ok(())
    }
}

impl Codec<Frame> for FrameCodec {}
