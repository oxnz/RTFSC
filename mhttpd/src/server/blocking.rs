use std::{
    io::BufReader,
    net::{SocketAddr, TcpListener, TcpStream},
};

use crate::http::{Header, Request, ResponseBuilder, SerDe};

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
        let response = ResponseBuilder::default()
            .version(crate::http::Version::Http11)
            .status(crate::http::Status::OK)
            .headers(vec![
                Header::Literal {
                    name: "content-type".to_string(),
                    value: "text/html".to_string(),
                },
                Header::Literal {
                    name: "content-length".to_string(),
                    value: body.len().to_string(),
                },
            ])
            .body(body)
            .build()?;
        tracing::debug!("response: {response:?}");
        response.write(reader.get_mut())?;
    }
    Ok(())
}
