#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Status(pub u16);

impl Status {
    // 1xx Informational
    pub const CONTINUE: Status = Status(100);
    pub const SWITCHING_PROTOCOLS: Status = Status(101);

    // 2xx Success
    pub const OK: Status = Status(200);
    pub const CREATED: Status = Status(201);
    pub const ACCEPTED: Status = Status(202);
    pub const NON_AUTHORITATIVE_INFORMATION: Status = Status(203);
    pub const NO_CONTENT: Status = Status(204);
    pub const RESET_CONTENT: Status = Status(205);
    pub const PARTIAL_CONTENT: Status = Status(206);

    // 3xx Redirection
    pub const MULTIPLE_CHOICES: Status = Status(300);
    pub const MOVED_PERMANENTLY: Status = Status(301);
    pub const FOUND: Status = Status(302);
    pub const SEE_OTHER: Status = Status(303);
    pub const NOT_MODIFIED: Status = Status(304);
    pub const USE_PROXY: Status = Status(305);
    pub const TEMPORARY_REDIRECT: Status = Status(307);

    // 4xx Client Error
    pub const BAD_REQUEST: Status = Status(400);
    pub const UNAUTHORIZED: Status = Status(401);
    pub const PAYMENT_REQUIRED: Status = Status(402);
    pub const FORBIDDEN: Status = Status(403);
    pub const NOT_FOUND: Status = Status(404);
    pub const METHOD_NOT_ALLOWED: Status = Status(405);
    pub const NOT_ACCEPTABLE: Status = Status(406);
    pub const PROXY_AUTHENTICATION_REQUIRED: Status = Status(407);
    pub const REQUEST_TIMEOUT: Status = Status(408);
    pub const CONFLICT: Status = Status(409);
    pub const GONE: Status = Status(410);
    pub const LENGTH_REQUIRED: Status = Status(411);
    pub const PRECONDITION_FAILED: Status = Status(412);
    pub const PAYLOAD_TOO_LARGE: Status = Status(413);
    pub const URI_TOO_LONG: Status = Status(414);
    pub const UNSUPPORTED_MEDIA_TYPE: Status = Status(415);
    pub const RANGE_NOT_SATISFIABLE: Status = Status(416);
    pub const EXPECTATION_FAILED: Status = Status(417);
    pub const TOO_MANY_REQUESTS: Status = Status(429);
    pub const MISDIRECTED_REQUEST: Status = Status(421);
    pub const UPGRADE_REQUIRED: Status = Status(426);

    // 5xx Server Error
    pub const INTERNAL_SERVER_ERROR: Status = Status(500);
    pub const NOT_IMPLEMENTED: Status = Status(501);
    pub const BAD_GATEWAY: Status = Status(502);
    pub const SERVICE_UNAVAILABLE: Status = Status(503);
    pub const GATEWAY_TIMEOUT: Status = Status(504);
    pub const HTTP_VERSION_NOT_SUPPORTED: Status = Status(505);

    pub fn reason_phrase(&self) -> &'static str {
        match self.0 {
            // 1xx Informational
            100 => "Continue",
            101 => "Switching Protocols",

            // 2xx Success
            200 => "OK",
            201 => "Created",
            202 => "Accepted",
            203 => "Non-Authoritative Information",
            204 => "No Content",
            205 => "Reset Content",
            206 => "Partial Content",

            // 3xx Redirection
            300 => "Multiple Choices",
            301 => "Moved Permanently",
            302 => "Found",
            303 => "See Other",
            304 => "Not Modified",
            305 => "Use Proxy",
            307 => "Temporary Redirect",

            // 4xx Client Error
            400 => "Bad Request",
            401 => "Unauthorized",
            402 => "Payment Required",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            406 => "Not Acceptable",
            407 => "Proxy Authentication Required",
            408 => "Request Timeout",
            409 => "Conflict",
            410 => "Gone",
            411 => "Length Required",
            412 => "Precondition Failed",
            413 => "Payload Too Large",
            414 => "URI Too Long",
            415 => "Unsupported Media Type",
            416 => "Range Not Satisfiable",
            417 => "Expectation Failed",
            421 => "Misdirected Request",
            426 => "Upgrade Required",
            429 => "Too Many Requests",

            // 5xx Server Error
            500 => "Internal Server Error",
            501 => "Not Implemented",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            504 => "Gateway Timeout",
            505 => "HTTP Version Not Supported",

            // Unknown
            _ => "",
        }
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conv() {
        assert!(matches!(Status::from(Status::OK), Status(200)));
    }

    #[test]
    fn test_display() {
        assert_eq!("200", Status::OK.to_string());
    }
}
