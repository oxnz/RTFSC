use std::io::{BufRead, Write};

use crate::http::{Header, Method, Request, Response, SerDe, Version};

impl SerDe for Request {
    fn read<R: BufRead>(r: &mut R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let mut line = Vec::with_capacity(8 * 1024);
        let n = r.read_until(b'\n', &mut line)?;
        if n == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::WouldBlock,
                "EAGAIN",
            ));
        }
        let request_line = str::from_utf8(line.trim_ascii_end())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let (method, path, version) = match request_line.split_once(' ') {
            Some((method, rest)) => match rest.rsplit_once(' ') {
                Some((target, protocol)) => (
                    method.parse::<Method>()?,
                    target.to_string(),
                    protocol.parse::<Version>()?,
                ),
                None => todo!(),
            },
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "invalid request line",
                ));
            }
        };
        let mut headers = Vec::new();
        let mut content_length = 0;
        loop {
            line.clear();
            r.read_until(b'\n', &mut line)?;
            let s = str::from_utf8(&line.trim_ascii_end())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            if s.is_empty() {
                break;
            }
            let header: Header = s.parse()?;
            if let Header::Literal { name, value } = &header {
                if name == "content-length" {
                    content_length = value
                        .parse()
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                }
            }

            headers.push(header);
        }
        let body = if content_length != 0 {
            let mut content = vec![0; content_length];
            r.read_exact(&mut content)?;
            Some(content)
        } else {
            None
        };
        Ok(Self {
            method,
            path,
            version,
            headers,
            body,
        })
    }

    fn write<W: Write>(&self, stream: &mut W) -> std::io::Result<()> {
        write!(stream, "{} {} {}\r\n", self.method, self.path, self.version)?;
        for header in &self.headers {
            match header {
                Header::Pseudo { name, value } | Header::Literal { name, value } => {
                    write!(stream, "{}: {}\r\n", name, value)?;
                }
            }
        }
        stream.write_all(b"\r\n")?;
        if let Some(content) = &self.body {
            stream.write_all(&content)?;
        }
        Ok(())
    }
}

impl SerDe for Response {
    fn read<R: BufRead>(r: &mut R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    fn write<W: Write>(&self, stream: &mut W) -> std::io::Result<()> {
        // status line
        if let Some(reason_phrase) = &self.reason_phrase {
            write!(
                stream,
                "{} {} {}\r\n",
                self.version, self.status_code, reason_phrase
            )?;
        } else {
            write!(stream, "{} {}\r\n", self.version, self.status_code)?;
        }

        // headers
        for header in &self.headers {
            write!(stream, "{:?}\r\n", header)?;
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
    use crate::http::{Method, Response, SerDe};

    use super::*;

    #[test]
    fn test_request_read() {
        let input = b"POST /submit HTTP/1.1\r\nHost: example.com\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: 14\r\n\r\ncomment=hello!";
        let request = Request::read(&mut std::io::Cursor::new(input)).unwrap();
        println!("{request:?}");
        assert_eq!(request.method, Method::POST);
        assert_eq!(request.path, "/submit");
        assert_eq!(request.headers.len(), 3);
        let mut buffer = Vec::new();
        request.write(&mut buffer).unwrap();
        assert_eq!(input.len(), buffer.len());
    }

    #[test]
    fn test_response_write() {
        let body = b"it works!".to_vec();
        let response = Response {
            version: crate::http::Version::Http11,
            status_code: crate::http::StatusCode::Ok,
            reason_phrase: None,
            headers: vec![
                Header::Literal {
                    name: "content-type".to_string(),
                    value: "text/plain".to_string(),
                },
                Header::Literal {
                    name: "content-length".to_string(),
                    value: body.len().to_string(),
                },
            ],
            body: Some(body),
        };
        let mut buffer = Vec::new();
        response.write(&mut buffer).unwrap();
        let s = String::from_utf8(buffer).unwrap();
        assert_eq!(
            s,
            "HTTP/1.1 200\r\ncontent-type: text/plain\r\ncontent-length: 9\r\n\r\nit works!"
        );
    }
}
