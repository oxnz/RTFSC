use std::str::FromStr;

#[derive(Debug)]
pub enum Scheme {
    Http,
    Https,
    Custom(String),
}

impl FromStr for Scheme {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "http" => Ok(Self::Http),
            "https" => Ok(Self::Https),
            _ => Ok(Self::Custom(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Scheme;

    #[test]
    fn test_parse() {
        assert!(matches!("http".parse(), Ok(Scheme::Http)));
        assert!(matches!("https".parse(), Ok(Scheme::Https)));
        assert!(matches!("httpx".parse(), Ok(Scheme::Custom(s)) if s == "httpx"));
    }
}
