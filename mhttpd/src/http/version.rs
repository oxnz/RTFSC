use std::str::FromStr;

#[derive(Debug)]
pub enum Version {
    Http10,
    Http11,
    Http2,
    Http3,
}

impl FromStr for Version {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HTTP/1.0" => Ok(Self::Http10),
            "HTTP/1.1" => Ok(Self::Http11),
            "HTTP/2" | "HTTP/2.0" => Ok(Self::Http2),
            "HTTP/3" | "HTTP/3.0" => Ok(Self::Http3),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "unsupported protocol",
            )),
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Version::Http10 => "HTTP/1.0",
            Version::Http11 => "HTTP/1.1",
            Version::Http2 => "HTTP/2.0",
            Version::Http3 => "HTTP/3.0",
        };
        f.write_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let raw = "HTTP/1.1";
        let v: Version = raw.parse().unwrap();
        assert!(matches!(v, Version::Http11));
        assert_eq!(raw, v.to_string());
    }
}
