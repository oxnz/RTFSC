use std::io::{BufWriter, Write};

use crate::http::{
    Request,
    codec::{Decode, Encode},
};

#[derive(Debug)]
pub struct RequestCodec {
    max_len_limit: usize,
}

impl Default for RequestCodec {
    fn default() -> Self {
        Self {
            max_len_limit: usize::MAX,
        }
    }
}

impl Encode<Request> for RequestCodec {
    fn encode<W: std::io::Write>(
        &mut self,
        request: Request,
        stream: &mut W,
    ) -> std::io::Result<()> {
        let mut stream = BufWriter::new(stream);
        // for line in item.as_ref().lines() {
        //     let sep = b"\r\n";
        //     let len = line.len() + sep.len();
        //     if len <= self.max_len_limit {
        //         stream.write_all(line.as_bytes())?;
        //         stream.write_all(sep.as_slice())?;
        //     } else {
        //         return Err(std::io::Error::new(
        //             std::io::ErrorKind::InvalidData,
        //             "line too long",
        //         ))
        //     }
        // }
        stream.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codec() {
        let raw = "POST / HTTP/1.1\r\nContent-Type: text/csv\r\n\r\nabc,def\r\n";
        let mut stream = Vec::new();
        let mut codec = RequestCodec::default();

        assert_eq!(raw.as_bytes(), stream);
    }
}
