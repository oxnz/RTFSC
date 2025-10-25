use std::{
    io::BufReader,
    net::{SocketAddr, TcpListener, TcpStream},
};

use mhttpd::http::{
    Header, SerDe,
    v2::{Frame, Preface, flags},
};

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    serve().unwrap();
}

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

fn process(addr: SocketAddr, stream: TcpStream) -> std::io::Result<()> {
    tracing::info!("connection from {addr:?}");
    let mut stream = BufReader::new(stream);
    // --- 1. Read connection preface ---
    Preface::read(&mut stream)?;
    tracing::info!("Received HTTP/2 preface");
    let frame = Frame::read(&mut stream)?;
    tracing::info!("rcvd frame: {frame:x?}");
    // --- 4. Send a simple SETTINGS ACK ---
    let settings_frame = Frame::Settings {
        ack: 0,
        items: vec![],
    };
    settings_frame.write(stream.get_mut())?;
    tracing::info!("Sent SETTINGS");

    let frame = Frame::read(&mut stream)?;
    tracing::info!("recvd 2: {frame:x?}");

    let frame = Frame::read(&mut stream)?;
    tracing::info!("recvd 3: {frame:x?}");

    let header_frame = Frame::Headers {
        stream_id: 1,
        flags: flags::END_HEADERS,
        items: vec![Header::new(":status", "200")],
    };
    header_frame.write(stream.get_mut())?;
    tracing::info!("Sent HEADERS frame (200 OK)");

    let data_frame = Frame::Data {
        stream_id: 1,
        flags: flags::END_STREAM,
        data: b"it works!".to_vec(),
    };
    data_frame.write(stream.get_mut())?;
    tracing::info!("Sent data frame");

    let frame = Frame::read(&mut stream)?;
    tracing::info!("recvd 4: {frame:x?}");
    // 4a. HEADERS frame for 200 OK

    // For simplicity, we close after first frame
    tracing::info!("Done with connection\n");
    Ok(())
}

#[test]
fn test_get() {
    let output = std::process::Command::new("curl")
        .args([
            "--http2-prior-knowledge",
            "-v",
            "--silent",
            "127.0.0.1:8000",
        ])
        .output()
        .unwrap();
    let stdout = output.stdout;
    let stderr = output.stderr;
    unsafe {
        println!("stdout:\n{}", str::from_utf8_unchecked(&stdout));
        eprintln!("stderr:\n{}", str::from_utf8_unchecked(&stderr));
    }
}
