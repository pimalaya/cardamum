use base64::{prelude::BASE64_STANDARD, Engine};
use chrono::Utc;

#[derive(Clone, Debug, Default)]
pub struct Request {
    bytes: Vec<u8>,
}

impl Request {
    pub const PROPFIND: &str = "PROPFIND";
    pub const REPORT: &str = "REPORT";

    pub fn new(method: &str, uri: &str) -> Self {
        let mut bytes = Vec::new();

        bytes.extend(method.as_bytes());
        bytes.push(b' ');

        bytes.extend(uri.as_bytes());
        bytes.push(b' ');

        bytes.extend(b"HTTP/1.1\r\n");

        bytes.extend(b"Date: ");
        bytes.extend(Utc::now().format("%a, %d %b %Y %T").to_string().as_bytes());
        bytes.extend(b" GMT\r\n");

        bytes.extend(b"Content-Type: application/xml; charset=utf-8\r\n");

        Self { bytes }
    }

    pub fn propfind(uri: &str) -> Self {
        Self::new(Self::PROPFIND, uri)
    }

    pub fn report(uri: &str) -> Self {
        Self::new(Self::REPORT, uri)
    }

    pub fn basic_auth(mut self, user: &str, pass: &str) -> Self {
        let auth = BASE64_STANDARD.encode(format!("{user}:{pass}"));
        self.bytes.extend(b"Authorization: Basic ");
        self.bytes.extend(auth.as_bytes());
        self.bytes.extend(b"\r\n");
        self
    }

    pub fn depth(mut self, depth: &str) -> Self {
        self.bytes.extend(b"Depth: ");
        self.bytes.extend(depth.as_bytes());
        self.bytes.extend(b"\r\n");
        self
    }

    pub fn body(mut self, body: &str) -> Self {
        self.bytes.extend(b"Content-Length: ");
        self.bytes.extend(body.len().to_string().as_bytes());
        self.bytes.extend(b"\r\n\r\n");
        self.bytes.extend(body.as_bytes());
        self
    }
}

impl From<Request> for Vec<u8> {
    fn from(request: Request) -> Self {
        request.bytes
    }
}
