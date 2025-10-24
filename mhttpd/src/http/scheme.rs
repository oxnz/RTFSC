#[derive(Debug)]
pub enum Scheme {
    Http,
    Https,
    Custom(String),
}

impl From<&str> for Scheme {
    fn from(value: &str) -> Self {
        match value {
            "http" => Self::Http,
            "https" => Self::Https,
            _ => Self::Custom(value.to_string()),
        }
    }
}

impl std::fmt::Display for Scheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scheme::Http => write!(f, "http"),
            Scheme::Https => write!(f, "https"),
            Scheme::Custom(value) => write!(f, "{value}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Scheme;

    #[test]
    fn test_parse() {
        assert!(matches!("http".into(), Scheme::Http));
        assert!(matches!("https".into(), Scheme::Https));
        assert!(matches!("httpx".into(), Scheme::Custom(s) if s == "httpx"));
    }

    #[test]
    fn test_display() {
        assert_eq!(Scheme::Http.to_string(), "http");
        assert_eq!(Scheme::Https.to_string(), "https");
        assert_eq!(Scheme::Custom("httpx".to_string()).to_string(), "httpx");
    }
}
