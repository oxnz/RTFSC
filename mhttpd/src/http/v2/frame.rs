use crate::http::{SerDe, v2::settings::Setting};

#[derive(Debug)]
pub enum Frame {
    Data {
        stream_id: u32,
        flags: u8,
        payload: Vec<u8>,
    },
    Headers {
        stream_id: u32,
        flags: u8,
        items: Vec<(Vec<u8>, Vec<u8>)>,
    },
    Settings {
        flags: u8,
        items: Vec<Setting>,
    },
    WindowUpdate {
        stream_id: u32,
        increment: u32,
    },
}

impl SerDe for Frame {
    fn read<R: std::io::BufRead>(stream: &mut R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        // read header
        let mut header = [0u8; 9];
        stream.read_exact(&mut header)?;
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
            0x0 => {
                // data
                Ok(Self::Data {
                    stream_id,
                    flags,
                    payload,
                })
            }
            0x1 => {
                // HEADERS
                let mut decoder = hpack::Decoder::new();
                let items = decoder.decode(&payload).unwrap();
                Ok(Self::Headers {
                    stream_id,
                    flags,
                    items,
                })
            }
            0x04 => {
                let mut items = Vec::new();
                let n = payload.len() / 6;
                let mut stream = std::io::Cursor::new(payload);
                for _i in 0..n {
                    let setting = Setting::read(&mut stream)?;
                    items.push(setting);
                }
                Ok(Self::Settings { flags, items })
            }
            0x08 => {
                if payload.len() == 4 {
                    let raw = u32::from_be_bytes(payload.try_into().unwrap());
                    let increment = raw & 0x7FFF_FFFF; // mask off reserved bit
                    if increment == 0 {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "non-zero value expected",
                        ));
                    }
                    Ok(Self::WindowUpdate {
                        stream_id,
                        increment,
                    })
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "4 bytes expected",
                    ))
                }
            }
            _ => {
                tracing::info!("Other frame type 0x{:x}", frame_type);
                panic!();
            }
        }
    }

    fn write<W: std::io::Write>(&self, stream: &mut W) -> std::io::Result<()> {
        match self {
            Frame::Settings { flags, items } => {
                let len: u32 = items.len() as u32 * 6;
                stream.write_all(&len.to_be_bytes()[1..])?;
                stream.write_all(&[0x04])?; // type
                stream.write_all(&[*flags])?; // flags
                stream.write_all(&0u32.to_be_bytes())?; // stream_id
                for item in items {
                    item.write(stream)?;
                }
            }
            Frame::WindowUpdate {
                stream_id,
                increment,
            } => {
                let len: u32 = 4;
                stream.write_all(&len.to_be_bytes()[1..])?;
                stream.write_all(&[0x08])?; // type
                stream.write_all(&[0])?; // flags
                stream.write_all(&stream_id.to_be_bytes())?;
                stream.write_all(&increment.to_be_bytes())?;
            }
            Frame::Headers {
                stream_id,
                flags,
                items,
            } => {
                let mut encoder = hpack::Encoder::new();
                let payload = encoder.encode(items.iter().map(|item| (&item.0[..], &item.1[..])));
                let len: u32 = payload.len() as u32;
                stream.write_all(&len.to_be_bytes()[1..])?;
                stream.write_all(&[0x01])?; // type
                stream.write_all(&[*flags])?; // flags
                stream.write_all(&stream_id.to_be_bytes())?;
                stream.write_all(&payload)?;
            }
            Frame::Data {
                stream_id,
                flags,
                payload,
            } => {
                let len: u32 = payload.len() as u32;
                stream.write_all(&len.to_be_bytes()[1..])?;
                stream.write_all(&[0x00])?; // type
                stream.write_all(&[*flags])?; // flags
                stream.write_all(&stream_id.to_be_bytes())?;
                stream.write_all(&payload)?;
            }
        }
        Ok(())
    }
}

#[test]
fn test_serde_data() {
    let raw = [0, 0, 0, 0, 0, 0, 0, 0, 0];
    let frame = Frame::read(&mut std::io::Cursor::new(raw)).unwrap();
    println!("{frame:?}");
    let mut v = Vec::new();
    frame.write(&mut v).unwrap();
    println!("{v:?}");
    assert_eq!(raw, v.as_slice());
}

#[test]
fn test_serde_headers() {
    let raw = [
        0, 0, 30, 1, 5, 0, 0, 0, 1, 130, 134, 65, 138, 8, 157, 92, 11, 129, 112, 220, 120, 0, 7,
        132, 122, 136, 37, 182, 80, 195, 203, 186, 184, 127, 83, 3, 42, 47, 42,
    ];
    let frame = Frame::read(&mut std::io::Cursor::new(raw)).unwrap();
    println!("{frame:?}");
    let mut v = Vec::new();
    frame.write(&mut v).unwrap();
    println!("{v:?}");
    // assert_eq!(raw, v.as_slice());
}

#[test]
fn test_serde_settings() {
    let raw = [
        0, 0, 18, 4, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 100, 0, 4, 0, 160, 0, 0, 0, 2, 0, 0, 0, 0,
    ];
    let frame = Frame::read(&mut std::io::Cursor::new(raw)).unwrap();
    println!("{frame:?}");
    let mut v = Vec::new();
    frame.write(&mut v).unwrap();
    println!("{v:?}");
    assert_eq!(raw, v.as_slice());
}

#[test]
fn test_serde_window_update() {
    let raw = [0, 0, 4, 8, 0, 0, 0, 0, 0, 62, 127, 0, 1];
    let frame = Frame::read(&mut std::io::Cursor::new(raw)).unwrap();
    println!("{frame:?}");
    let mut v = Vec::new();
    frame.write(&mut v).unwrap();
    println!("{v:?}");
    assert_eq!(raw, v.as_slice());
}

#[test]
fn test_hpack() {
    let raw = [
        130, 134, 65, 138, 8, 157, 92, 11, 129, 112, 220, 120, 0, 7, 132, 122, 136, 37, 182, 80,
        195, 203, 186, 184, 127, 83, 3, 42, 47, 42,
    ];
    let mut decoder = hpack::Decoder::new();
    let items = decoder.decode(&raw).unwrap();
    let mut encoder = hpack::Encoder::new();
    let result = encoder.encode(items.iter().map(|(k, v)| (&k[..], &v[..])));
    assert_eq!(raw, result.as_slice());
}
