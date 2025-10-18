use std::str::FromStr;

#[derive(Debug)]
pub enum Header {
    // common header
    ContentType(String),
    ContentLength(usize),

    // request header
    Host(String),
    UserAgent(String),
    Accept(String),

    // response header
    Server(String),
    Date(String),
    CacheControl(String),

    // other
    Custom { name: String, value: String },
}

impl FromStr for Header {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(": ") {
            Some((name, value)) => match name {
                "Host" => Ok(Self::Host(value.to_string())),
                "Content-Type" => Ok(Self::ContentType(value.to_string())),
                "Content-Length" => {
                    Ok(Self::ContentLength(value.parse().map_err(|e| {
                        std::io::Error::new(std::io::ErrorKind::InvalidData, e)
                    })?))
                }
                _ => Ok(Self::Custom {
                    name: name.to_string(),
                    value: value.to_string(),
                }),
            },
            None => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "':' not found",
            )),
        }
    }
}

impl std::fmt::Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Header::ContentType(s) => write!(f, "Content-Type: {}", s),
            Header::ContentLength(n) => write!(f, "Content-Length: {n}"),
            Header::Host(_) => todo!(),
            Header::UserAgent(_) => todo!(),
            Header::Accept(_) => todo!(),
            Header::Server(_) => todo!(),
            Header::Date(_) => todo!(),
            Header::CacheControl(_) => todo!(),
            Header::Custom { name, value } => write!(f, "{}: {}", name, value),
        }
    }
}
