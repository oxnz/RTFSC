use std::net::{SocketAddr, TcpListener, TcpStream};

use crate::http::{Header, Request, Response, SerDe};

pub fn serve() -> std::io::Result<()> {
    let socket = TcpListener::bind("127.0.0.1:8000")?;
    loop {
        match socket.accept() {
            Ok((mut stream, remote_addr)) => {
                process(remote_addr, &mut stream)?;
                drop(stream);
            }
            Err(e) => tracing::error!("accept: {e:?}"),
        }
    }
    Ok(())
}

fn process(remote_addr: SocketAddr, stream: &mut TcpStream) -> std::io::Result<()> {
    let request = Request::read(stream)?;
    tracing::debug!("request: {request:?} from addr: {remote_addr:?}");
    let sample_html = r#"
    <html lang="en">
    <body>
        <h1>it works!</h1>
    </body>
    </html>
    "#;
    let response = Response::new(
        crate::http::Protocol::Http("1.1".to_string()),
        crate::http::StatusCode::Ok,
        None,
        vec![Header::ContentType("text/html".to_string())],
        Some(sample_html.as_bytes().to_vec()),
    );
    tracing::debug!("response: {response:?}");
    response.write(stream)?;
    Ok(())
}
