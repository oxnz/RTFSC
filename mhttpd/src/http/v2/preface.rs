use std::io::BufRead;

pub const PREFACE: &'static [u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

#[derive(Debug)]
pub struct Preface;

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
