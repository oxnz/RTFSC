use crate::http::{Response, v2::connection::Connection};

#[derive(Debug)]
pub struct Client {
    connection: Connection,
}

impl Client {
    pub async fn request() -> std::io::Result<Response> {
        todo!()
    }
}
