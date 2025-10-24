use bytes::BufMut;

use crate::http::{
    codec::{Decode, Encode},
    v2::{Frame, FrameType},
};

#[derive(Debug)]
pub struct FrameEncoder {}

impl Encode<Frame> for FrameEncoder {
    fn encode(&mut self, item: Frame, stream: &mut bytes::BytesMut) -> std::io::Result<()> {
        let header_len = 9;
        match item {
            Frame::Settings { flags, items } => {
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
                let mut encoder = hpack::Encoder::new();
                let payload = encoder.encode(items.iter().map(|item| (&item.0[..], &item.1[..])));
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
            Frame::RstStream { error_code } => {
                let payload_len = 4u32;
                stream.reserve(header_len + payload_len as usize);
                let stream_id = 0u32;
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
