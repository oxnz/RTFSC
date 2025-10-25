use std::str::FromStr;

pub enum Header {
    // V2
    Pseudo { name: String, value: String },
    // other
    Literal { name: String, value: String },
}

impl Header {
    pub fn new<K: Into<String>, V: Into<String>>(name: K, value: V) -> Self {
        let mut name: String = name.into();
        name.make_ascii_lowercase();
        let value = value.into();
        if name.starts_with(':') {
            Self::Literal { name, value }
        } else {
            Self::Pseudo { name, value }
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Header::Pseudo { name, value } => &name,
            Header::Literal { name, value } => &name,
        }
    }

    pub fn value(&self) -> &str {
        match self {
            Header::Pseudo { name, value } => &value,
            Header::Literal { name, value } => &value,
        }
    }
}

impl FromStr for Header {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(": ") {
            Some((name, value)) => Ok(Self::new(name, value)),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "':' not found",
            )),
        }
    }
}

impl std::fmt::Debug for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Header::Literal { name, value } => write!(f, "{}: {}", name, value),
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
        assert!(matches!(header, Header::Literal {name, value} if name == "content-type"));

        let header = Header::from_str("Content-Length: 123").unwrap();
        assert!(matches!(header, Header::Literal {name, value} if name == "content-length"));

        assert!(Header::from_str("Invalid").is_err());
    }
}
