use std::{
    io::BufReader,
    net::{SocketAddr, TcpListener, TcpStream},
};

use crate::http::{Header, Request, Response, SerDe};

pub fn serve() -> std::io::Result<()> {
    let socket = TcpListener::bind("127.0.0.1:8000")?;
    loop {
        match socket.accept() {
            Ok((stream, remote_addr)) => {
                if let Err(e) = process(remote_addr, stream) {
                    tracing::error!("error: {e:?}");
                }
            }
            Err(e) => tracing::error!("accept: {e:?}"),
        }
    }
    Ok(())
}

fn process(remote_addr: SocketAddr, stream: TcpStream) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream);
    loop {
        let request = Request::read(&mut reader)?;
        tracing::debug!("request: {request:?} from addr: {remote_addr:?}");
        let body = br#"<html lang="en"><body><h1>it works!</h1></body></html>"#.to_vec();
        let response = Response::new(
            crate::http::Protocol::Http("1.1".to_string()),
            crate::http::StatusCode::Ok,
            None,
            vec![
                Header::ContentType("text/html".to_string()),
                Header::ContentLength(body.len()),
            ],
            Some(body),
        );
        tracing::debug!("response: {response:?}");
        response.write(reader.get_mut())?;
    }
    Ok(())
}
