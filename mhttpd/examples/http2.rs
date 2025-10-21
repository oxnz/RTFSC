use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
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
    let mut stream = BufReader::new(stream);
    // --- 1. Read connection preface ---
    Preface::read(&mut stream)?;
    tracing::info!("Received HTTP/2 preface");
    let frame = Frame::read(&mut stream)?;
    tracing::info!("rcvd frame: {frame:x?}");
    // --- 4. Send a simple SETTINGS ACK ---
    let ack_frame = [0, 0, 0, 0x4, 0x1, 0, 0, 0, 0]; // SETTINGS frame, ACK flag
    let stream = stream.get_mut();
    stream.write_all(&ack_frame)?;
    stream.flush()?;
    tracing::info!("Sent SETTINGS ACK");

    // 4a. HEADERS frame for 200 OK
    let headers_payload = vec![
        // HPACK-encoded headers (very minimal)
        0x88, // :status = 200 (from static table)
    ];
    let headers_len = headers_payload.len();
    let mut headers_frame = Vec::with_capacity(9 + headers_len);
    headers_frame.push(((headers_len >> 16) & 0xFF) as u8);
    headers_frame.push(((headers_len >> 8) & 0xFF) as u8);
    headers_frame.push((headers_len & 0xFF) as u8);
    headers_frame.push(0x1); // HEADERS
    headers_frame.push(0x5); // END_HEADERS | END_STREAM
    headers_frame.push(0); // stream id MSB
    headers_frame.push(0);
    headers_frame.push(0);
    headers_frame.push(1); // stream id LSB = 1
    headers_frame.extend_from_slice(&headers_payload);

    stream.write_all(&headers_frame)?;
    stream.flush()?;
    tracing::info!("Sent HEADERS frame (200 OK)");

    // Optional: send DATA frame (body)
    let body = b"Hello from Rust h2c!\n";
    let body_len = body.len();
    let mut data_frame = Vec::with_capacity(9 + body_len);
    data_frame.push(((body_len >> 16) & 0xFF) as u8);
    data_frame.push(((body_len >> 8) & 0xFF) as u8);
    data_frame.push((body_len & 0xFF) as u8);
    data_frame.push(0x0); // DATA frame
    data_frame.push(0x1); // END_STREAM
    data_frame.push(0);
    data_frame.push(0);
    data_frame.push(0);
    data_frame.push(1); // stream id = 1
    data_frame.extend_from_slice(body);

    stream.write_all(&data_frame)?;
    stream.flush()?;
    tracing::info!("Sent DATA frame with body");

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
        println!("stdout:\n{:?}", str::from_utf8_unchecked(&stdout));
        eprintln!("stderr:\n{:?}", str::from_utf8_unchecked(&stderr));
    }
}

#[derive(Debug)]
struct Preface;

impl Preface {
    pub fn read<R: BufRead>(stream: &mut R) -> std::io::Result<Self> {
        let mut preface = [0u8; 24];
        const PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
        stream.read_exact(&mut preface)?;
        if preface != PREFACE {
            tracing::error!("{:?}", unsafe { str::from_utf8_unchecked(&preface) });
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid HTTP/2 preface",
            ));
        }
        Ok(Self)
    }
}

#[derive(Debug)]
pub enum FrameType {
    Data = 0x00,
    Headers = 0x01,
    Priority = 0x02,
    RstStream = 0x03,
    Settings = 0x04,
    PushPromise = 0x05,
    Ping = 0x06,
    GoAway = 0x07,
    WindowUpdate = 0x08,
    Continuation = 0x09,
}

impl From<u8> for FrameType {
    fn from(value: u8) -> Self {
        unsafe { std::mem::transmute::<u8, Self>(value) }
    }
}

#[derive(Debug)]
struct Frame {
    len: u32,
    r#type: FrameType,
    flags: u8,
    stream_id: u32,
    payload: Vec<u8>,
}

impl Frame {
    pub fn read<R: BufRead>(stream: &mut R) -> std::io::Result<Self> {
        // read header
        let mut header = [0u8; 9];
        let n = stream.read(&mut header)?;
        let len = ((header[0] as u32) << 16) | ((header[1] as u32) << 8) | (header[2] as u32);
        let frame_type = header[3];
        let flags = header[4];
        let stream_id = ((header[5] as u32 & 0x7F) << 24)
            | ((header[6] as u32) << 16)
            | ((header[7] as u32) << 8)
            | (header[8] as u32);

        //  Read payload ---
        let mut payload = vec![0u8; len as usize];
        stream.read_exact(&mut payload)?;
        match frame_type {
            0x4 => {
                // SETTINGS
                tracing::info!("Received SETTINGS frame");
                // Normally you parse key-value settings pairs here
            }
            0x1 => {
                // HEADERS
                tracing::info!("Received HEADERS frame ({} bytes)", payload.len());
                tracing::info!(
                    "Raw header payload: {:x?}",
                    &payload[..payload.len().min(32)]
                );
            }
            _ => tracing::info!("Other frame type 0x{:x}", frame_type),
        }
        Ok(Self {
            len,
            r#type: frame_type.into(),
            flags,
            stream_id,
            payload,
        })
    }
}
