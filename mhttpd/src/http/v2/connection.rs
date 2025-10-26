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
        loop {
            let frame = self.transport.read_frame().await?;
            tracing::info!("read frame: {frame:?}");
            match frame {
                Frame::Data {
                    stream_id,
                    end_stream,
                    data,
                    pad_len: _,
                } => {
                    let stream = self
                        .streams
                        .entry(stream_id)
                        .or_insert(Stream::new(stream_id));
                    stream.request_builder.extend_body(&data);
                    if end_stream {
                        stream.state = crate::http::v2::stream::State::Closed;
                        return std::mem::take(&mut stream.request_builder)
                            .build()
                            .map(|x| (stream_id, x));
                    }
                }
                Frame::Headers {
                    stream_id,
                    flags,
                    items,
                } => {
                    let stream = self
                        .streams
                        .entry(stream_id)
                        .or_insert(Stream::new(stream_id));
                    for header in items {
                        stream.request_builder.add_header(header);
                    }
                    if 0 != flags & flags::END_STREAM {
                        stream.state = crate::http::v2::stream::State::HalfClosedRemote;
                        return std::mem::take(&mut stream.request_builder)
                            .build()
                            .map(|x| (stream_id, x));
                    }
                }
                Frame::Priority {
                    stream_id,
                    exclusive,
                    stream_dependency,
                    weight,
                } => {
                    todo!()
                }
                Frame::RstStream {
                    stream_id,
                    error_code,
                } => {
                    if let Some(stream) = self.streams.get_mut(&stream_id) {
                        stream.state = crate::http::v2::stream::State::Closed;
                    }
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::ConnectionReset,
                        "stream reset",
                    ));
                }
                Frame::Settings { ack, items } => {
                    if ack {
                        tracing::info!("settings acked");
                        assert!(items.is_empty());
                    } else {
                        self.transport
                            .send_frame(Frame::Settings {
                                ack: true,
                                items: vec![],
                            })
                            .await?;
                    }
                }
                Frame::PushPromise {
                    stream_id,
                    end_headers,
                    headers,
                    pad_len,
                    promised_stream_id,
                } => {
                    todo!()
                }
                Frame::Ping { ack, opaque_data } => {
                    if ack {
                        tracing::info!("ping acked, data: {opaque_data:?}");
                    } else {
                        self.transport
                            .send_frame(Frame::Ping {
                                ack: true,
                                opaque_data,
                            })
                            .await?;
                    }
                }
                Frame::GoAway {
                    last_stream_id,
                    error_code,
                    debug_data,
                } => {
                    todo!()
                }
                Frame::WindowUpdate {
                    stream_id,
                    increment,
                } => {
                    tracing::error!("todo, ignore windowUpdate for now");
                }
                Frame::Continuation {
                    end_headers,
                    stream_id,
                    headers,
                } => {
                    let stream = self
                        .streams
                        .entry(stream_id)
                        .or_insert(Stream::new(stream_id));
                    for header in headers {
                        stream.request_builder.add_header(header);
                    }
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
        let mut headers = response.headers;
        headers.insert(
            0,
            crate::http::Header::new(":status", response.status_code.to_string()),
        );
        let header_frame = Frame::Headers {
            stream_id,
            flags: flags::END_HEADERS,
            items: headers,
        };
        self.transport.send_frame(header_frame).await?;
        let data_frame: Frame = Frame::Data {
            stream_id,
            end_stream: true,
            data: response.body.unwrap_or_default(),
            pad_len: None,
        };
        self.transport.send_frame(data_frame).await?;
        Ok(())
    }
}
