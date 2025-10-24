use bytes::BufMut;

use crate::http::codec::Encode;

#[derive(Debug)]
pub struct LineCodec {
    search_offset: usize,
    max_len_limit: usize,
}

impl Default for LineCodec {
    fn default() -> Self {
        Self {
            search_offset: 0,
            max_len_limit: usize::MAX,
        }
    }
}

impl<T: AsRef<str>> Encode<T> for LineCodec {
    fn encode(&mut self, item: T, stream: &mut bytes::BytesMut) -> std::io::Result<()> {
        let line = item.as_ref();
        let sep = b"\r\n";
        let len = line.len() + sep.len();
        if len <= self.max_len_limit {
            stream.reserve(len);
            stream.put(line.as_bytes());
            stream.put(sep.as_slice());
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "line too long",
            ))
        }
    }
}

impl super::Decode<String> for LineCodec {
    fn decode(&mut self, stream: &mut bytes::BytesMut) -> std::io::Result<Option<String>> {
        match stream[self.search_offset..]
            .iter()
            .position(|b| *b == b'\n')
        {
            Some(offset) => {
                let newline_offset = offset + self.search_offset;
                self.search_offset = 0;
                let line = stream.split_to(newline_offset + 1);
                let line = if line.ends_with(b"\r\n") {
                    &line[..line.len() - 2]
                } else {
                    &line[..line.len() - 1]
                };
                let s = String::from_utf8(line.to_vec())
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                Ok(Some(s))
            }
            None => {
                self.search_offset = stream.len();
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use crate::http::codec::Decode;

    use super::*;

    #[test]
    fn test_codec() {
        let raw = "POST / HTTP/1.1\r\nContent-Type: text/csv\r\n\r\nabc,def\r\n";
        let mut stream = BytesMut::new();
        let mut codec = LineCodec::default();
        for line in raw.lines() {
            codec.encode(line, &mut stream).unwrap();
        }
        assert_eq!(raw, stream);
        while let Ok(Some(line)) = codec.decode(&mut stream) {
            println!("{line:?}");
        }
    }
}
