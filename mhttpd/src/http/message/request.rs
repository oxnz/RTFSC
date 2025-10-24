use std::io::{BufRead, Write};

use crate::http::{Header, Method, Scheme, SerDe, Version};

#[derive(Debug, Default)]
pub struct RequestBuilder {
    method: Option<Method>,
    scheme: Option<Scheme>,
    authority: Option<String>,
    path: Option<String>,
    headers: Vec<Header>,
    body: Option<Vec<u8>>,
}

impl RequestBuilder {
    pub fn set_method(&mut self, value: Method) {
        self.method = Some(value)
    }

    pub fn set_scheme(&mut self, scheme: Scheme) {
        self.scheme = Some(scheme);
    }

    pub fn set_authority(&mut self, authority: String) {
        self.authority = Some(authority)
    }

    pub fn set_path(&mut self, path: String) {
        self.path = Some(path)
    }

    pub fn add_header<K: Into<String>, V: Into<String>>(&mut self, name: K, value: V) {
        self.headers.push(Header::Custom {
            name: name.into(),
            value: value.into(),
        });
    }

    pub fn extend_body(&mut self, data: &[u8]) {
        self.body = match self.body.take() {
            Some(mut body) => {
                body.extend_from_slice(data);
                Some(body)
            }
            None => Some(data.to_vec()),
        }
    }

    pub fn build(self) -> std::io::Result<Request> {
        Ok(Request::new(
            self.method.unwrap(),
            self.path.unwrap(),
            Version::Http("2.0".to_string()),
            self.headers,
            self.body,
        ))
    }
}

#[derive(Debug)]
pub struct Request {
    method: Method,
    path: String,
    protocol: Version,
    headers: Vec<Header>,
    body: Option<Vec<u8>>,
}

impl Request {
    pub fn new(
        method: Method,
        target: String,
        protocol: Version,
        headers: Vec<Header>,
        body: Option<Vec<u8>>,
    ) -> Self {
        Self {
            method,
            path: target,
            protocol,
            headers,
            body,
        }
    }
}

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
        let (method, target, protocol) = match request_line.split_once(' ') {
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
            if let Header::ContentLength(n) = header {
                content_length = n;
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
            path: target,
            protocol,
            headers,
            body,
        })
    }

    fn write<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::http::SerDe;

    use super::*;

    #[test]
    fn test_request_read() {
        let input = b"POST /submit HTTP/1.1\r\nHost: example.com\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: 14\r\n\r\ncomment=hello!";
        let request = Request::read(&mut std::io::Cursor::new(input)).unwrap();
        println!("{request:?}");
        assert_eq!(request.method, Method::POST);
        assert_eq!(request.path, "/submit");
        assert_eq!(request.headers.len(), 3);
    }
}
