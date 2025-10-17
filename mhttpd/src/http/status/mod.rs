#[repr(u16)]
#[derive(Debug)]
pub enum StatusCode {
    Ok = 200,
    Created = 201,
    NotFound = 404,
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatusCode::Ok => f.write_str("200"),
            StatusCode::Created => f.write_str("201"),
            StatusCode::NotFound => f.write_str("404"),
        }
    }
}
