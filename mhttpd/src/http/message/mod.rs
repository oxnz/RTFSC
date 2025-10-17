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
        let reader = BufReader::new(r);
        let mut lines = reader.lines();
        let request_line = lines.next().unwrap()?;
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
        for line in lines {
            let line = line?;
            if line.is_empty() {
                break;
            }
            let header: Header = line.parse()?;
            headers.push(header);
        }
        Ok(Self {
            method,
            target,
            protocol,
            headers,
            body: None,
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
        writeln!(
            w,
            "{} {} {}",
            self.protocol,
            self.status_code,
            self.reason_phrase.as_ref().unwrap_or(&"".to_string())
        )?;
        // headers
        for header in &self.headers {
            writeln!(w, "{}", header)?;
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
fn test_response_write() {
    let sample_html = r#"
    <html lang="en">
    <body>
        <h1>it works!</h1>
    </body>
    </html>
    "#;
    let response = Response::new(
        crate::http::Protocol::Http("1.1".to_string()),
        crate::http::StatusCode::Ok,
        None,
        vec![Header::ContentType("text/html".to_string())],
        Some(sample_html.as_bytes().to_vec()),
    );
    let mut buf = Vec::new();
    response.write(&mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    println!("{s}");
}
