use std::str::FromStr;

#[derive(Debug)]
pub enum Method {
    Head,
    Options,
    Get,
    POST,
    PUT,
    DELETE,
    CONNECT,
}

impl FromStr for Method {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HEAD" => Ok(Self::Head),
            "OPTIONS" => Ok(Self::Options),
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::POST),
            "PUT" => Ok(Self::PUT),
            "DELETE" => Ok(Self::DELETE),
            "CONNECT" => Ok(Self::CONNECT),
            _ => todo!(),
        }
    }
}
