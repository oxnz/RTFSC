use std::io::{BufRead, BufWriter, Write};

use crate::http::{Header, SerDe, StatusCode, Version};

#[derive(Debug)]
pub struct Response {
    protocol: Version,
    status_code: StatusCode,
    reason_phrase: Option<String>,
    headers: Vec<Header>,
    pub(crate) body: Option<Vec<u8>>,
}

impl Response {
    pub fn new(
        protocol: Version,
        status_code: StatusCode,
        reason_phrase: Option<String>,
        headers: Vec<Header>,
        body: Option<Vec<u8>>,
    ) -> Self {
        Self {
            protocol,
            status_code,
            reason_phrase,
            headers,
            body,
        }
    }
}

impl SerDe for Response {
    fn read<R: BufRead>(r: &mut R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    fn write<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        let mut stream = BufWriter::new(w);
        // status line
        if let Some(reason_phrase) = &self.reason_phrase {
            write!(
                stream,
                "{} {} {}\r\n",
                self.protocol, self.status_code, reason_phrase
            )?;
        } else {
            write!(stream, "{} {}\r\n", self.protocol, self.status_code)?;
        }

        // headers
        for header in &self.headers {
            write!(stream, "{}\r\n", header)?;
        }
        stream.write(b"\r\n")?;
        if let Some(body) = self.body.as_ref() {
            stream.write_all(&body)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::http::SerDe;

    use super::*;

    #[test]
    fn test_response_write() {
        let body = b"it works!".to_vec();
        let response = Response::new(
            crate::http::Version::Http("1.1".to_string()),
            crate::http::StatusCode::Ok,
            None,
            vec![
                Header::ContentType("text/plain".to_string()),
                Header::ContentLength(body.len()),
            ],
            Some(body),
        );
        let mut buf = Vec::new();
        response.write(&mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert_eq!(
            s,
            "HTTP/1.1 200\r\nContent-Type: text/plain\r\nContent-Length: 9\r\n\r\nit works!"
        );
    }
}
