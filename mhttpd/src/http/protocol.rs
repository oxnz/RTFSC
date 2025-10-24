use std::str::FromStr;

#[derive(Debug)]
pub enum Protocol {
    Http(String),
}

impl FromStr for Protocol {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HTTP/1.1" => Ok(Self::Http("1.1".to_string())),
            "HTTP/2.0" => Ok(Self::Http("2.0".to_string())),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "unsupported protocol",
            )),
        }
    }
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Http(version) => write!(f, "HTTP/{}", version),
        }
    }
}
