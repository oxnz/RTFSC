use std::collections::HashMap;

use tokio::net::TcpStream;

use crate::http::{
    Request, Response,
    v2::{Frame, Transport, flags, stream::Stream},
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

    pub async fn read_request(&mut self) -> std::io::Result<(u32, Request)> {
        tracing::info!("read request");
        loop {
            let frame = self.transport.read_frame().await?;
            tracing::info!("read frame: {frame:?}");
            match frame {
                crate::http::v2::Frame::Data {
                    stream_id,
                    flags,
                    data,
                } => {
                    let stream = self
                        .streams
                        .entry(stream_id)
                        .or_insert(Stream::new(stream_id));
                    stream.request_builder.extend_body(&data);
                    if 0 != flags & flags::END_STREAM {
                        return std::mem::take(&mut stream.request_builder)
                            .build()
                            .map(|x| (stream_id, x));
                    }
                }
                crate::http::v2::Frame::Headers {
                    stream_id,
                    flags,
                    items,
                } => {
                    let stream = self
                        .streams
                        .entry(stream_id)
                        .or_insert(Stream::new(stream_id));
                    for header in items {
                        let key = header.name();
                        let value = header.value();
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
                    if 0 != flags & flags::END_STREAM {
                        return std::mem::take(&mut stream.request_builder)
                            .build()
                            .map(|x| (stream_id, x));
                    }
                }
                crate::http::v2::Frame::RstStream { error_code } => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::ConnectionReset,
                        "stream reset",
                    ));
                }
                crate::http::v2::Frame::Settings { flags, items } => {
                    if 0 != flags & flags::ACK {
                        tracing::info!("settings acked");
                    } else {
                        self.transport
                            .send_frame(&Frame::Settings {
                                flags: flags::ACK,
                                items: vec![],
                            })
                            .await?;
                    }
                }
                crate::http::v2::Frame::WindowUpdate {
                    stream_id,
                    increment,
                } => {
                    tracing::error!("todo, ignore windowUpdate for now");
                }
            }
        }
    }

    pub async fn send_response(
        &mut self,
        stream_id: u32,
        response: Response,
    ) -> std::io::Result<()> {
        tracing::info!("send reponse: {response:?}");
        let header_frame = Frame::Headers {
            stream_id,
            flags: flags::END_HEADERS,
            items: response.headers,
        };
        self.transport.send_frame(&header_frame).await?;
        let data_frame: Frame = Frame::Data {
            stream_id,
            flags: flags::END_STREAM,
            data: response.body.unwrap_or_default(),
        };
        self.transport.send_frame(&data_frame).await?;
        Ok(())
    }
}
