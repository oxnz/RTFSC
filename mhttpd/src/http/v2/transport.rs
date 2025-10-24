use std::collections::HashMap;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::http::RequestBuilder;

#[derive(Debug)]
pub struct Transport {
    stream: TcpStream,
    streams: HashMap<u32, RequestBuilder>,
}

impl From<TcpStream> for Transport {
    fn from(value: TcpStream) -> Self {
        Self {
            stream: value,
            streams: HashMap::default(),
        }
    }
}

const PREFACE: &'static [u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

impl Transport {
    pub async fn exchange_preface(&mut self) -> std::io::Result<()> {
        let mut preface = [0u8; PREFACE.len()];
        self.stream.read_exact(&mut preface).await.unwrap();
        assert_eq!(preface, PREFACE);
        let client_settings = self.read_frame().await?;
        assert_eq!(client_settings.r#type, 0x04);
        let server_settings = Frame {
            r#type: 0x04,
            flags: 0,
            stream_id: 0,
            payload: vec![],
        };
        self.send_frame(&server_settings).await?;
        tracing::debug!("preface exchanged");
        Ok(())
    }

    pub async fn read_frame(&mut self) -> std::io::Result<Frame> {
        let mut header = [0u8; 9];
        self.stream.read_exact(&mut header).await?;
        let len = ((header[0] as u32) << 16) | ((header[1] as u32) << 8) | (header[2] as u32);
        let frame_type = header[3];
        let flags = header[4];
        let stream_id = ((header[5] as u32 & 0x7F) << 24)
            | ((header[6] as u32) << 16)
            | ((header[7] as u32) << 8)
            | (header[8] as u32);
        let mut payload = vec![0u8; len as usize];
        self.stream.read_exact(&mut payload).await?;
        Ok(Frame {
            r#type: frame_type,
            flags,
            stream_id,
            payload,
        })
    }

    pub async fn send_frame(&mut self, frame: &Frame) -> std::io::Result<()> {
        let len: u32 = frame.payload.len() as u32;
        self.stream.write_all(&len.to_be_bytes()[1..]).await?;
        self.stream.write_all(&[frame.r#type]).await?; // type
        self.stream.write_all(&[frame.flags]).await?; // flags
        self.stream
            .write_all(&frame.stream_id.to_be_bytes())
            .await?; // stream_id
        self.stream.write_all(&frame.payload).await
    }
}

#[derive(Debug)]
pub struct Frame {
    pub(crate) r#type: u8,
    pub(crate) flags: u8,
    pub(crate) stream_id: u32,
    pub(crate) payload: Vec<u8>,
}
