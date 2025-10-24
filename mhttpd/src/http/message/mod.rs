use std::io::{BufRead, BufWriter, Write};

use crate::http::{Header, Method, Protocol, StatusCode};

pub trait SerDe {
    fn read<R: BufRead>(r: &mut R) -> std::io::Result<Self>
    where
        Self: Sized;
    fn write<W: Write>(&self, w: &mut W) -> std::io::Result<()>;
}

#[derive(Debug)]
pub struct Request {
    method: Method,
    target: String,
    protocol: Protocol,
    headers: Vec<Header>,
    body: Option<Vec<u8>>,
}

impl Request {
    pub fn new(
        method: Method,
        target: String,
        protocol: Protocol,
        headers: Vec<Header>,
        body: Option<Vec<u8>>,
    ) -> Self {
        Self {
            method,
            target,
            protocol,
            headers,
            body,
        }
    }
}

#[derive(Debug)]
pub struct Response {
    protocol: Protocol,
    status_code: StatusCode,
    reason_phrase: Option<String>,
    headers: Vec<Header>,
    pub(crate) body: Option<Vec<u8>>,
}

impl Response {
    pub fn new(
        protocol: Protocol,
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
                    protocol.parse::<Protocol>()?,
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
            target,
            protocol,
            headers,
            body,
        })
    }

    fn write<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        todo!()
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

#[test]
fn test_request_read() {
    let input = b"POST /submit HTTP/1.1\r\nHost: example.com\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: 14\r\n\r\ncomment=hello!";
    let request = Request::read(&mut std::io::Cursor::new(input)).unwrap();
    println!("{request:?}");
    assert_eq!(request.method, Method::POST);
    assert_eq!(request.target, "/submit");
    assert_eq!(request.headers.len(), 3);
}

#[test]
fn test_response_write() {
    let body = b"it works!".to_vec();
    let response = Response::new(
        crate::http::Protocol::Http("1.1".to_string()),
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
