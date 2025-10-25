use std::collections::HashMap;

use hpack::{Decoder, Encoder};
use tokio::net::TcpStream;

use crate::http::{
    Request, Response,
    v2::{FrameType, Transport, flags, stream::Stream},
};

#[derive(Debug)]
pub struct Connection {
    transport: Transport,
    streams: HashMap<u32, Stream>,
}

impl From<TcpStream> for Connection {
    fn from(value: TcpStream) -> Self {
        Self {
            transport: value.into(),
            streams: HashMap::default(),
        }
    }
}

impl Connection {
    pub async fn exchange_preface(&mut self) -> std::io::Result<()> {
        self.transport.exchange_preface().await
    }

    pub async fn read_frame(&mut self) -> std::io::Result<Frame> {
        self.transport.read_frame().await
    }

    pub async fn send_frame(&mut self, frame: &Frame) -> std::io::Result<()> {
        self.transport.send_frame(frame).await
    }

    pub async fn read_request(&mut self) -> std::io::Result<Request> {
        tracing::info!("read request");
        loop {
            let frame = self.read_frame().await?;
            tracing::info!("read frame: {frame:?}");
            match FrameType::from(frame.r#type) {
                FrameType::Data => {
                    let stream = self
                        .streams
                        .entry(frame.stream_id)
                        .or_insert(Stream::new(frame.stream_id));
                    stream.request_builder.extend_body(frame.payload.as_slice());
                    if 0 != frame.flags & flags::END_STREAM {
                        return std::mem::take(&mut stream.request_builder).build();
                    }
                }
                FrameType::Headers => {
                    let stream = self
                        .streams
                        .entry(frame.stream_id)
                        .or_insert(Stream::new(frame.stream_id));
                    let mut decoder = Decoder::new();
                    for (k, v) in decoder.decode(&frame.payload).unwrap() {
                        let key = str::from_utf8(&k).unwrap();
                        let value = str::from_utf8(&v).unwrap();
                        match key {
                            ":method" => {
                                stream.request_builder.set_method(value.parse().unwrap());
                            }
                            ":scheme" => {
                                stream.request_builder.set_scheme(value.into());
                            }
                            ":authority" => {
                                stream.request_builder.set_authority(value.to_string());
                            }
                            ":path" => {
                                stream.request_builder.set_path(value.to_string());
                            }
                            _ => {
                                stream.request_builder.add_header(key, value);
                            }
                        }
                    }
                    if 0 != frame.flags & flags::END_STREAM {
                        return std::mem::take(&mut stream.request_builder).build();
                    }
                }
                FrameType::Settings => {
                    tracing::info!("ignore settings");
                }
                FrameType::WindowUpdate => {
                    tracing::info!("window update: {frame:?}");
                }
                _ => {
                    tracing::error!("unknown frame: {frame:?}");
                }
            }
        }
        todo!()
    }

    pub async fn send_response(&mut self, response: Response) -> std::io::Result<()> {
        tracing::info!("send reponse: {response:?}");
        let mut encoder = Encoder::new();
        let payload = encoder.encode([(b":status".as_slice(), b"200".as_slice())]);
        let header_frame = Frame {
            r#type: 0x01,
            flags: flags::END_HEADERS,
            stream_id: 1,
            payload,
        };
        self.send_frame(&header_frame).await?;
        let data_frame = Frame {
            r#type: 0x00,
            flags: flags::END_STREAM,
            stream_id: 1,
            payload: response.body.unwrap_or_default(),
        };
        self.send_frame(&data_frame).await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Frame {
    pub(crate) r#type: u8,
    pub(crate) flags: u8,
    pub(crate) stream_id: u32,
    pub(crate) payload: Vec<u8>,
}
