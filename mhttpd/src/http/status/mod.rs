#[derive(Debug, Clone, Copy)]
pub enum StatusCode {
    Continue,
    SwitchingProtocols,
    Ok,
    Created,
    Accepted,
    NonAuthoritativeInformation,
    NoContent,
    ResetContent,
    PartialContent,
    MultipleChoices,
    MovedPermanently,
    Found,
    SeeOther,
    NotModified,
    UseProxy,
    TemporaryRedirect,
    BadRequest,
    Unauthorized,
    PaymentRequired,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    NotAccetable,
    ProxyAuthenticationRequired,
    RequestTimeout,
    Conflict,
    Gone,
    LengthRequired,
    PreconditionFailed,
    PayloadTooLarge,
    URITooLong,
    UnsupportedMediaType,
    RangeNotSatisfiable,
    ExpectationFailed,
    TooManyRequests,
    MisdirectedRequest,
    UpgradeRequired,
    InternalServerError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
    GatewayTimeout,
    HTTPVersionNotSupported,
    Unknown(u16),
}

impl From<u16> for StatusCode {
    fn from(value: u16) -> Self {
        match value {
            100 => Self::Continue,
            101 => Self::SwitchingProtocols,
            200 => Self::Ok,
            201 => Self::Created,
            202 => Self::Accepted,
            203 => Self::NonAuthoritativeInformation,
            204 => Self::NoContent,
            205 => Self::ResetContent,
            206 => Self::PartialContent,
            300 => Self::MultipleChoices,
            301 => Self::MovedPermanently,
            302 => Self::Found,
            303 => Self::SeeOther,
            304 => Self::NotModified,
            305 => Self::UseProxy,
            307 => Self::TemporaryRedirect,
            400 => Self::BadRequest,
            401 => Self::Unauthorized,
            402 => Self::PaymentRequired,
            403 => Self::Forbidden,
            404 => Self::NotFound,
            405 => Self::MethodNotAllowed,
            406 => Self::NotAccetable,
            407 => Self::ProxyAuthenticationRequired,
            408 => Self::RequestTimeout,
            409 => Self::Conflict,
            410 => Self::Gone,
            411 => Self::LengthRequired,
            412 => Self::PreconditionFailed,
            413 => Self::PayloadTooLarge,
            414 => Self::URITooLong,
            415 => Self::UnsupportedMediaType,
            416 => Self::RangeNotSatisfiable,
            417 => Self::ExpectationFailed,
            419 => Self::TooManyRequests,
            421 => Self::MisdirectedRequest,
            426 => Self::UpgradeRequired,
            500 => Self::InternalServerError,
            501 => Self::NotImplemented,
            502 => Self::BadGateway,
            503 => Self::ServiceUnavailable,
            504 => Self::GatewayTimeout,
            505 => Self::HTTPVersionNotSupported,
            _ => Self::Unknown(value),
        }
    }
}

impl From<StatusCode> for u16 {
    fn from(status: StatusCode) -> Self {
        match status {
            StatusCode::Continue => 100,
            StatusCode::SwitchingProtocols => 101,
            StatusCode::Ok => 200,
            StatusCode::Created => 201,
            StatusCode::Accepted => 202,
            StatusCode::NonAuthoritativeInformation => 203,
            StatusCode::NoContent => 204,
            StatusCode::ResetContent => 205,
            StatusCode::PartialContent => 206,
            StatusCode::MultipleChoices => 300,
            StatusCode::MovedPermanently => 301,
            StatusCode::Found => 302,
            StatusCode::SeeOther => 303,
            StatusCode::NotModified => 304,
            StatusCode::UseProxy => 305,
            StatusCode::TemporaryRedirect => 307,
            StatusCode::BadRequest => 400,
            StatusCode::Unauthorized => 401,
            StatusCode::PaymentRequired => 402,
            StatusCode::Forbidden => 403,
            StatusCode::NotFound => 404,
            StatusCode::MethodNotAllowed => 405,
            StatusCode::NotAccetable => 406,
            StatusCode::ProxyAuthenticationRequired => 407,
            StatusCode::RequestTimeout => 408,
            StatusCode::Conflict => 409,
            StatusCode::Gone => 410,
            StatusCode::LengthRequired => 411,
            StatusCode::PreconditionFailed => 412,
            StatusCode::PayloadTooLarge => 413,
            StatusCode::URITooLong => 414,
            StatusCode::UnsupportedMediaType => 415,
            StatusCode::RangeNotSatisfiable => 416,
            StatusCode::ExpectationFailed => 417,
            StatusCode::TooManyRequests => 419,
            StatusCode::MisdirectedRequest => 421,
            StatusCode::UpgradeRequired => 426,
            StatusCode::InternalServerError => 500,
            StatusCode::NotImplemented => 501,
            StatusCode::BadGateway => 502,
            StatusCode::ServiceUnavailable => 503,
            StatusCode::GatewayTimeout => 504,
            StatusCode::HTTPVersionNotSupported => 505,
            StatusCode::Unknown(value) => value,
        }
    }
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", u16::from(*self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conv() {
        assert!(matches!(u16::from(StatusCode::Ok), 200));
        assert!(matches!(u16::from(StatusCode::Unknown(999)), 999));
        assert!(matches!(StatusCode::from(200u16), StatusCode::Ok));
        assert!(matches!(StatusCode::from(999u16), StatusCode::Unknown(value) if value == 999));
    }

    #[test]
    fn test_display() {
        assert_eq!("200", StatusCode::Ok.to_string());
        assert_eq!("999", StatusCode::Unknown(999).to_string());
    }
}
