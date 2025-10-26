use std::vec;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
};

use crate::http::{
    codec::{Decode, Encode},
    v2::{Frame, frame::FrameCodec, settings::Setting},
};

/**
 * socket
 * codec
 */
pub struct Transport {
    reader: BufReader<OwnedReadHalf>,
    writer: BufWriter<OwnedWriteHalf>,
    frame_codec: FrameCodec,
}

impl std::fmt::Debug for Transport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Transport")
            .field("reader", &self.reader)
            .field("writer", &self.writer)
            .finish()
    }
}

impl From<TcpStream> for Transport {
    fn from(value: TcpStream) -> Self {
        let (read_half, write_half) = value.into_split();
        Self {
            frame_codec: Default::default(),
            reader: BufReader::new(read_half),
            writer: BufWriter::new(write_half),
        }
    }
}

impl Transport {
    pub async fn exchange_preface(&mut self) -> std::io::Result<()> {
        let mut preface = [0u8; PREFACE.len()];
        self.reader.read_exact(&mut preface).await?;
        assert_eq!(preface, PREFACE);
        let client_settings = self.read_frame().await?;
        tracing::info!("client settings: {:?}", client_settings);
        assert!(
            matches!(client_settings, Frame::Settings { ack, items } if !ack && !items.is_empty())
        );
        let server_settings = Frame::Settings {
            ack: false,
            items: vec![
                Setting::HeaderTableSize(1 << 14),
                Setting::EnablePush(false),
                Setting::MaxConcurrentStreams(128),
                Setting::InitialWindowSize(1 << 23),
                Setting::MaxFrameSize(1 << 14),
                Setting::MaxHeaderListSize(1 << 12),
            ],
        };
        self.send_frame(server_settings).await?;
        tracing::debug!("preface exchanged");
        Ok(())
    }

    pub async fn read_frame(&mut self) -> std::io::Result<Frame> {
        let mut header = [0u8; 9];
        self.reader.read_exact(&mut header).await?;
        let payload_len =
            (((header[0] as u32) << 16) | ((header[1] as u32) << 8) | (header[2] as u32)) as usize;
        if payload_len > 0 {
            let mut buffer = Vec::with_capacity(9 + payload_len);
            buffer.extend_from_slice(&header);
            unsafe {
                buffer.set_len(9 + payload_len);
            }
            self.reader.read_exact(&mut buffer[header.len()..]).await?;
            tracing::debug!("read raw frame: {buffer:?}");
            self.frame_codec.decode(&mut std::io::Cursor::new(buffer))
        } else {
            tracing::debug!("read raw frame: {header:?}");
            self.frame_codec.decode(&mut std::io::Cursor::new(&header))
        }
    }

    pub async fn send_frame(&mut self, frame: Frame) -> std::io::Result<()> {
        let mut buffer = Vec::with_capacity(9);
        self.frame_codec.encode(frame, &mut buffer)?;
        tracing::debug!("send raw frame: {buffer:?}");
        self.writer.write_all(&buffer).await?;
        self.writer.flush().await
    }
}

const PREFACE: &'static [u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

#[cfg(test)]
mod tests {
    #[test]
    fn test_upgrade() {
        let raw = r#"
GET / HTTP/1.1
Host: localhost:8000
User-Agent: curl/8.7.1
Accept: */*
Connection: Upgrade, HTTP2-Settings
Upgrade: h2c
HTTP2-Settings: AAMAAABkAAQAoAAAAAIAAAAA


        "#;
        let v = [
            71, 69, 84, 32, 47, 32, 72, 84, 84, 80, 47, 49, 46, 49, 13, 10, 72, 111, 115, 116, 58,
            32, 108, 111,
        ];
        let s = str::from_utf8(v.as_slice());
        println!("{s:?}");
    }
}
