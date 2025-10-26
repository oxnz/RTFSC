use crate::http::{Header, Method, Scheme, Version};

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

    pub fn add_header(&mut self, header: Header) {
        match &header {
            Header::Pseudo { name, value } => match name.as_str() {
                ":method" => {
                    self.set_method(value.parse().expect("invalid method"));
                }
                ":scheme" => {
                    self.set_scheme(Scheme::from(value.as_ref()));
                }
                ":authority" => {
                    self.set_authority(value.to_string());
                }
                ":path" => {
                    self.set_path(value.to_string());
                }
                _ => {
                    self.headers.push(header);
                }
            },
            Header::Literal { name, value } => {
                self.headers.push(header);
            }
        }
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
        Ok(Request {
            method: self.method.expect("no method"),
            path: self.path.expect("no path"),
            version: Version::Http("2.0".to_string()),
            headers: self.headers,
            body: self.body,
        })
    }
}

#[derive(Debug)]
pub struct Request {
    pub(crate) method: Method,
    pub(crate) path: String,
    pub(crate) version: Version,
    pub(crate) headers: Vec<Header>,
    pub(crate) body: Option<Vec<u8>>,
}
