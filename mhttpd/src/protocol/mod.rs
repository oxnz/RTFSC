use crate::http::{Request, Response};

mod v2;

pub trait Protocol {
    fn recv_request(&mut self) -> std::io::Result<Request>;
    fn send_response(&mut self, response: &Response) -> std::io::Result<()>;
}
