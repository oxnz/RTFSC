use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Method {
    HEAD,
    OPTIONS,
    GET,
    POST,
    PUT,
    DELETE,
    CONNECT,
    PATCH,
    TRACE,
}

impl FromStr for Method {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HEAD" => Ok(Self::HEAD),
            "OPTIONS" => Ok(Self::OPTIONS),
            "GET" => Ok(Self::GET),
            "POST" => Ok(Self::POST),
            "PUT" => Ok(Self::PUT),
            "DELETE" => Ok(Self::DELETE),
            "CONNECT" => Ok(Self::CONNECT),
            "PATCH" => Ok(Self::PATCH),
            "TRACE" => Ok(Self::TRACE),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid HTTP method",
            )),
        }
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Method::HEAD => write!(f, "HEAD"),
            Method::OPTIONS => write!(f, "OPTIONS"),
            Method::GET => write!(f, "GET"),
            Method::POST => write!(f, "POST"),
            Method::PUT => write!(f, "PUT"),
            Method::DELETE => write!(f, "DELETE"),
            Method::CONNECT => write!(f, "CONNECT"),
            Method::PATCH => write!(f, "PATCH"),
            Method::TRACE => write!(f, "TRACE"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert!(matches!("HEAD".parse(), Ok(Method::HEAD)));
        assert!(matches!("OPTIONS".parse(), Ok(Method::OPTIONS)));
        assert!(matches!("GET".parse(), Ok(Method::GET)));
        assert!(matches!("POST".parse(), Ok(Method::POST)));
        assert!(matches!("PUT".parse(), Ok(Method::PUT)));
        assert!(matches!("DELETE".parse(), Ok(Method::DELETE)));
        assert!(matches!("CONNECT".parse(), Ok(Method::CONNECT)));
        assert!(matches!("PATCH".parse(), Ok(Method::PATCH)));
        assert!(matches!("TRACE".parse(), Ok(Method::TRACE)));

        // Test invalid methods
        assert!(matches!("HEADx".parse::<Method>(), Err(_)));
        assert!(matches!("".parse::<Method>(), Err(_)));
        assert!(matches!("INVALID".parse::<Method>(), Err(_)));
    }

    #[test]
    fn test_display() {
        assert_eq!(Method::HEAD.to_string(), "HEAD");
        assert_eq!(Method::OPTIONS.to_string(), "OPTIONS");
        assert_eq!(Method::GET.to_string(), "GET");
        assert_eq!(Method::POST.to_string(), "POST");
        assert_eq!(Method::PUT.to_string(), "PUT");
        assert_eq!(Method::DELETE.to_string(), "DELETE");
        assert_eq!(Method::CONNECT.to_string(), "CONNECT");
        assert_eq!(Method::PATCH.to_string(), "PATCH");
        assert_eq!(Method::TRACE.to_string(), "TRACE");
    }

    #[test]
    fn test_roundtrip() {
        // Test that parsing and then displaying gives the original string
        let methods = [
            "HEAD", "OPTIONS", "GET", "POST", "PUT", "DELETE", "CONNECT", "PATCH", "TRACE",
        ];
        for method in methods {
            let parsed: Method = method.parse().unwrap();
            let displayed = parsed.to_string();
            assert_eq!(method, displayed);
        }
    }
}
