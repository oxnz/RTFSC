use crate::http::Header;

pub enum State {
    Idle,
    Open,
    HalfClosedLocal,
    HalfClosedRemote,
    Closed,
}

pub struct Stream {
    id: u32,
    state: State,
    headers: Vec<Header>,
    body: Vec<u8>,
}
