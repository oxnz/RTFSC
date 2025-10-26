use std::str::FromStr;

#[derive(Debug)]
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
            Self::Pseudo { name, value }
        } else {
            Self::Literal { name, value }
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Header::Pseudo { name, value: _ } | Header::Literal { name, value: _ } => &name,
        }
    }

    pub fn value(&self) -> &str {
        match self {
            Header::Pseudo { name: _, value } | Header::Literal { name: _, value } => &value,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let header = Header::from_str("Content-Type: text/html").unwrap();
        assert!(
            matches!(header, Header::Literal {name, value} if name == "content-type" && value == "text/html")
        );

        let header = Header::from_str("Content-Length: 123").unwrap();
        assert!(
            matches!(header, Header::Literal {name, value} if name == "content-length" && value == "123")
        );

        assert!(Header::from_str("Invalid").is_err());
    }

    #[test]
    fn test_new() {
        let header = Header::new(":status", "200");
        let h2 = Header::Pseudo {
            name: ":status".to_string(),
            value: "200".to_string(),
        };
        println!("{header:?}, {h2:?}");
    }
}
