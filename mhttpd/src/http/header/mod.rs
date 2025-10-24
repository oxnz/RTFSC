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

    // V2
    Pseudo { name: String, value: String },
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
            Header::Host(s) => write!(f, "Host: {}", s),
            Header::UserAgent(s) => write!(f, "User-Agent: {}", s),
            Header::Accept(s) => write!(f, "Accept: {}", s),
            Header::Server(s) => write!(f, "Server: {}", s),
            Header::Date(s) => write!(f, "Date: {}", s),
            Header::CacheControl(s) => write!(f, "Cache-Control: {}", s),
            Header::Custom { name, value } => write!(f, "{}: {}", name, value),
            Header::Pseudo { name, value } => write!(f, "{}: {}", name, value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let header = Header::from_str("Content-Type: text/html").unwrap();
        assert!(matches!(header, Header::ContentType(s) if s == "text/html"));

        let header = Header::from_str("Content-Length: 123").unwrap();
        assert!(matches!(header, Header::ContentLength(n) if n == 123));

        let header = Header::from_str("Host: example.com").unwrap();
        assert!(matches!(header, Header::Host(s) if s == "example.com"));

        let header = Header::from_str("X-Custom: value").unwrap();
        assert!(
            matches!(header, Header::Custom { name, value } if name == "X-Custom" && value == "value")
        );

        assert!(Header::from_str("Invalid").is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(
            Header::ContentType("text/html".to_string()).to_string(),
            "Content-Type: text/html"
        );
        assert_eq!(
            Header::ContentLength(123).to_string(),
            "Content-Length: 123"
        );
        assert_eq!(
            Header::Host("example.com".to_string()).to_string(),
            "Host: example.com"
        );
        assert_eq!(
            Header::UserAgent("Mozilla".to_string()).to_string(),
            "User-Agent: Mozilla"
        );
        assert_eq!(Header::Accept("*/*".to_string()).to_string(), "Accept: */*");
        assert_eq!(
            Header::Server("mhttpd".to_string()).to_string(),
            "Server: mhttpd"
        );
        assert_eq!(
            Header::Date("2023-01-01".to_string()).to_string(),
            "Date: 2023-01-01"
        );
        assert_eq!(
            Header::CacheControl("no-cache".to_string()).to_string(),
            "Cache-Control: no-cache"
        );
        assert_eq!(
            Header::Custom {
                name: "X-Custom".to_string(),
                value: "value".to_string(),
            }
            .to_string(),
            "X-Custom: value"
        );
        assert_eq!(
            Header::Pseudo {
                name: ":method".to_string(),
                value: "GET".to_string(),
            }
            .to_string(),
            ":method: GET"
        );
    }
}
