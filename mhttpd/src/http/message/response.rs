use crate::http::{Header, Status, Version};

#[derive(Debug)]
pub struct Response {
    pub(crate) version: Version,
    pub(crate) status: Status,
    pub(crate) headers: Vec<Header>,
    pub(crate) body: Option<Vec<u8>>,
}

#[derive(Debug, Default)]
pub struct ResponseBuilder {
    version: Option<Version>,
    status: Option<Status>,
    headers: Option<Vec<Header>>,
    body: Option<Vec<u8>>,
}

impl ResponseBuilder {
    pub fn version(mut self, version: Version) -> Self {
        self.version = Some(version);
        self
    }

    pub fn status(mut self, status: Status) -> Self {
        self.status = Some(status);
        self
    }

    pub fn headers(mut self, headers: Vec<Header>) -> Self {
        self.headers = Some(headers);
        self
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    pub fn build(self) -> std::io::Result<Response> {
        let version = self.version.ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "missing version",
        ))?;
        let status_code = self.status.ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "missing status",
        ))?;
        let headers = self.headers.ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "missing headers",
        ))?;
        Ok(Response {
            version,
            status: status_code,
            headers,
            body: self.body,
        })
    }
}
