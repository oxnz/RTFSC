use std::collections::HashMap;

use hpack::{Decoder, Encoder};
use tokio::net::TcpStream;

use crate::http::{
    Request, RequestBuilder, Response,
    v2::{Connection, FrameType, flags},
};

#[derive(Debug)]
pub struct Transport {
    connection: Connection,
    streams: HashMap<u32, RequestBuilder>,
}

impl From<TcpStream> for Transport {
    fn from(value: TcpStream) -> Self {
        Self {
            connection: value.into(),
            streams: HashMap::default(),
        }
    }
}

impl Transport {
    pub async fn exchange_preface(&mut self) -> std::io::Result<()> {
        self.connection.exchange_preface().await
    }

    pub async fn read_frame(&mut self) -> std::io::Result<Frame> {
        self.connection.read_frame().await
    }

    pub async fn send_frame(&mut self, frame: &Frame) -> std::io::Result<()> {
        self.connection.send_frame(frame).await
    }

    pub async fn read_request(&mut self) -> std::io::Result<Request> {
        tracing::info!("read request");
        loop {
            let frame = self.read_frame().await?;
            tracing::info!("read frame: {frame:?}");
            match FrameType::from(frame.r#type) {
                FrameType::Data => {
                    let request_builder = self
                        .streams
                        .entry(frame.stream_id)
                        .or_insert(Default::default());
                    request_builder.extend_body(frame.payload.as_slice());
                    if 0 != frame.flags & flags::END_STREAM {
                        return std::mem::take(request_builder).build();
                    }
                }
                FrameType::Headers => {
                    let request_builder = self
                        .streams
                        .entry(frame.stream_id)
                        .or_insert(Default::default());
                    let mut decoder = Decoder::new();
                    for (k, v) in decoder.decode(&frame.payload).unwrap() {
                        let key = str::from_utf8(&k).unwrap();
                        let value = str::from_utf8(&v).unwrap();
                        match key {
                            ":method" => {
                                request_builder.set_method(value.parse().unwrap());
                            }
                            ":scheme" => {
                                request_builder.set_scheme(value.into());
                            }
                            ":authority" => {
                                request_builder.set_authority(value.to_string());
                            }
                            ":path" => {
                                request_builder.set_path(value.to_string());
                            }
                            _ => {
                                request_builder.add_header(key, value);
                            }
                        }
                    }
                    if 0 != frame.flags & flags::END_STREAM {
                        return std::mem::take(request_builder).build();
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
