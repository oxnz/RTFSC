use std::vec;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::http::v2::{Frame, settings::Setting};

/**
 * socket
 * codec
 */
pub struct Transport {
    stream: TcpStream,
    hpack_encoder: hpack::Encoder<'static>,
    hpack_decoder: hpack::Decoder<'static>,
}

impl std::fmt::Debug for Transport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Transport")
            .field("stream", &self.stream)
            .finish()
    }
}

impl From<TcpStream> for Transport {
    fn from(value: TcpStream) -> Self {
        Self {
            stream: value,
            hpack_encoder: hpack::Encoder::new(),
            hpack_decoder: hpack::Decoder::new(),
        }
    }
}

impl Transport {
    pub async fn exchange_preface(&mut self) -> std::io::Result<()> {
        let mut preface = [0u8; PREFACE.len()];
        self.stream.read_exact(&mut preface).await.unwrap();
        assert_eq!(preface, PREFACE);
        let client_settings = self.read_frame().await?;
        tracing::info!("client settings: {:?}", client_settings);
        // assert_eq!(client_settings.r#type, 0x04);

        let server_settings = Frame::Settings {
            ack: 0,
            items: vec![
                Setting::MaxConcurrentStreams(10),
                Setting::InitialWindowSize(10485760),
            ],
        };
        self.send_frame(&server_settings).await?;
        tracing::debug!("preface exchanged");
        Ok(())
    }

    pub async fn read_frame(&mut self) -> std::io::Result<Frame> {
        let mut header = [0u8; 9];
        self.stream.read_exact(&mut header).await?;
        let len = ((header[0] as u32) << 16) | ((header[1] as u32) << 8) | (header[2] as u32);
        let mut payload = vec![0u8; len as usize];
        self.stream.read_exact(&mut payload).await?;
        Frame::decode(&header, &payload, &mut self.hpack_decoder)
    }

    pub async fn send_frame(&mut self, frame: &Frame) -> std::io::Result<()> {
        tracing::info!("send frame: {frame:?}");
        let mut buffer = Vec::with_capacity(9);
        frame.encode(&mut buffer, &mut self.hpack_encoder)?;
        self.stream.write_all(&buffer).await
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
