use crate::http::{RequestBuilder, v2::Frame};

#[derive(Debug)]
pub enum State {
    Idle,
    Open,
    HalfClosedLocal,
    HalfClosedRemote,
    Closed,
}

#[derive(Debug)]
pub struct Stream {
    id: u32,
    state: State,
    pub(crate) request_builder: RequestBuilder,
}

impl Stream {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            state: State::Idle,
            request_builder: RequestBuilder::default(),
        }
    }

    pub fn process(&mut self, frame: &Frame) -> std::io::Result<()> {
        Ok(())
    }
}
