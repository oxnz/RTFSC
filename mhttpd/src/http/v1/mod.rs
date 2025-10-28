use crate::http::{Request, RequestBuilder};

mod codec;

pub struct RequestEncoder {
    buffer: Vec<u8>,
    request_builder: Option<RequestBuilder>,
}

impl RequestEncoder {
    pub fn encode(&mut self, request: Request) -> std::io::Result<()> {
        if self.request_builder.is_some() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::WouldBlock,
                "inflight",
            ));
        } else {
            self.request_builder.insert(RequestBuilder::default());
            Ok(())
        }
    }
}
