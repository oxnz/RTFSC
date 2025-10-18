use std::io::{BufRead, BufReader, Read, Write};

use crate::http::{Header, Method, Protocol, StatusCode};

pub trait SerDe {
    fn read<R: Read>(r: &mut R) -> std::io::Result<Self>
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

#[derive(Debug)]
pub struct Response {
    protocol: Protocol,
    status_code: StatusCode,
    reason_phrase: Option<String>,
    headers: Vec<Header>,
    body: Option<Vec<u8>>,
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
    fn read<R: Read>(r: &mut R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let mut reader = BufReader::new(r);
        let mut line = Vec::with_capacity(8 * 1024);
        reader.read_until(b'\n', &mut line)?;
        if line.ends_with(b"\r\n") {
            line.truncate(line.len() - 2);
        }
        let request_line = str::from_utf8(&line)
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
            None => todo!(),
        };
        let mut headers = Vec::new();
        let mut content_length = 0;
        loop {
            line.clear();
            reader.read_until(b'\n', &mut line)?;
            if line.ends_with(b"\r\n") {
                line.truncate(line.len() - 2);
            }
            if line.is_empty() {
                break;
            }
            let s = str::from_utf8(&line)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            tracing::debug!("header: {s}");
            let header: Header = s.parse()?;
            if let Header::ContentLength(n) = header {
                content_length = n;
            }

            headers.push(header);
        }
        let body = if content_length != 0 {
            let mut content = vec![0; content_length];
            reader.read_exact(&mut content)?;
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
    fn read<R: Read>(r: &mut R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    fn write<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        // status line
        if let Some(reason_phrase) = &self.reason_phrase {
            write!(
                w,
                "{} {} {}\r\n",
                self.protocol, self.status_code, reason_phrase
            )?;
        } else {
            write!(w, "{} {}\r\n", self.protocol, self.status_code)?;
        }

        // headers
        for header in &self.headers {
            write!(w, "{}\r\n", header)?;
        }
        if let Some(body) = self.body.as_ref() {
            w.write_all(&body)?;
        } else {
            w.write(b"\r\n")?;
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
    let response = Response::new(
        crate::http::Protocol::Http("1.1".to_string()),
        crate::http::StatusCode::Ok,
        None,
        vec![Header::ContentType("text/plain".to_string())],
        Some("it works!".as_bytes().to_vec()),
    );
    let mut buf = Vec::new();
    response.write(&mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    assert_eq!(
        s,
        "HTTP/1.1 200\r\nContent-Type: text/plain\r\n\r\nit works!"
    );
}
