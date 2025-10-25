use bytes::{Buf, BufMut};

use crate::http::{
    codec::{Decode, Encode},
    v2::{ErrorCode, Frame, FrameType, flags, settings::Setting},
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

impl Encode<Frame> for FrameCodec {
    fn encode(&mut self, item: Frame, stream: &mut bytes::BytesMut) -> std::io::Result<()> {
        let header_len = 9;
        match item {
            Frame::Settings { ack: flags, items } => {
                let payload_len: u32 = items.len() as u32 * 6;
                stream.reserve(header_len + payload_len as usize);
                stream.put(&payload_len.to_be_bytes()[1..]);
                stream.put_u8(FrameType::Settings.into());
                stream.put_u8(flags);
                stream.put_u32(0); // stream_id
                for setting in items {
                    stream.put_u16(setting.identifier());
                    stream.put_u32(setting.value());
                }
            }
            Frame::WindowUpdate {
                stream_id,
                increment,
            } => {
                let payload_len: u32 = 4;
                stream.reserve(header_len + payload_len as usize);
                stream.put(&payload_len.to_be_bytes()[1..]);
                stream.put_u8(FrameType::WindowUpdate.into());
                stream.put_u8(0); // flags
                stream.put_u32(stream_id);
                stream.put_u32(increment);
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
                let payload_len: u32 = payload.len() as u32;
                stream.reserve(header_len + payload_len as usize);
                stream.put(&payload_len.to_be_bytes()[1..]);
                stream.put_u8(FrameType::Headers.into());
                stream.put_u8(flags);
                stream.put_u32(stream_id);
                stream.put(payload.as_slice());
            }
            Frame::Data {
                stream_id,
                flags,
                data,
            } => {
                let payload_len: u32 = data.len() as u32;
                stream.reserve(header_len + payload_len as usize);
                stream.put(&payload_len.to_be_bytes()[1..]);
                stream.put_u8(FrameType::Data.into());
                stream.put_u8(flags);
                stream.put_u32(stream_id);
                stream.put(data.as_slice());
            }
            Frame::RstStream {
                stream_id,
                error_code,
            } => {
                let payload_len = 4u32;
                stream.reserve(header_len + payload_len as usize);
                let data = u32::from(error_code);
                stream.put(&payload_len.to_be_bytes()[1..]);
                stream.put_u8(FrameType::RstStream.into());
                stream.put_u8(0); // flags
                stream.put_u32(stream_id);
                stream.put(data.to_be_bytes().as_slice());
            }
        }
        Ok(())
    }
}

impl Decode<Frame> for FrameCodec {
    fn decode(&mut self, stream: &mut bytes::BytesMut) -> std::io::Result<Option<Frame>> {
        if let Some(header) = stream.get(0..9) {
            let len = (((header[0] as u32) << 16) | ((header[1] as u32) << 8) | (header[2] as u32))
                as usize;
            let frame_type: FrameType = header[3].into();
            let flags = header[4];
            let stream_id = ((header[5] as u32 & 0x7F) << 24)
                | ((header[6] as u32) << 16)
                | ((header[7] as u32) << 8)
                | (header[8] as u32);

            if let Some(payload) = stream.get(0..len) {
                let frame = match frame_type {
                    FrameType::Data => {
                        let data = if 0 == flags & flags::PADDED {
                            payload.to_vec()
                        } else {
                            let pad_length = payload[0] as usize;
                            payload[1..len - pad_length].to_vec()
                        };
                        Frame::Data {
                            stream_id,
                            flags,
                            data,
                        }
                    }
                    FrameType::Headers => {
                        let items = self.hpack_decoder.decode(&payload).unwrap();
                        Frame::Headers {
                            stream_id,
                            flags,
                            items: vec![],
                        }
                    }
                    FrameType::RstStream => Frame::RstStream {
                        stream_id,
                        error_code: ErrorCode::from(u32::from_be_bytes(
                            *payload.first_chunk::<4>().unwrap(),
                        )),
                    },
                    FrameType::Settings => {
                        let mut items = Vec::new();
                        let n = payload.len() / 6;
                        let mut stream = std::io::Cursor::new(payload);
                        for _i in 0..n {
                            let identifier = stream.get_u16();
                            let value = stream.get_u32();
                            let setting = Setting::new(identifier, value);
                            items.push(setting);
                        }
                        Frame::Settings { ack: flags, items }
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
                            Frame::WindowUpdate {
                                stream_id,
                                increment,
                            }
                        } else {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "4 bytes expected",
                            ));
                        }
                    }
                    _ => {
                        tracing::info!("Other frame type {:?}", frame_type);
                        panic!();
                    }
                };
                Ok(Some(frame))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}
