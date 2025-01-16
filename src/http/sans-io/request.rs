use base64::{prelude::BASE64_STANDARD, Engine};
use chrono::Utc;

pub const CR: u8 = b'\r';
pub const LF: u8 = b'\n';
pub const SP: u8 = b' ';
pub const CRLF: [u8; 2] = [CR, LF];

#[derive(Clone, Debug, Default)]
pub struct Request {
    bytes: Vec<u8>,
}

impl Request {
    pub const PROPFIND: &str = "PROPFIND";
    pub const REPORT: &str = "REPORT";

    pub fn new(method: &str, uri: &str, version: &str) -> Self {
        let mut bytes = Vec::new();

        bytes.extend(method.as_bytes());
        bytes.push(SP);
        bytes.extend(uri.as_bytes());
        bytes.push(SP);
        bytes.extend(b"HTTP/");
        bytes.extend(version.as_bytes());
        bytes.extend(CRLF);

        bytes.extend(b"Date: ");
        bytes.extend(Utc::now().format("%a, %d %b %Y %T").to_string().as_bytes());
        bytes.extend(b" GMT");
        bytes.extend(CRLF);

        bytes.extend(b"Content-Type: application/xml; charset=utf-8");
        bytes.extend(CRLF);

        Self { bytes }
    }

    pub fn propfind(uri: &str, version: &str) -> Self {
        Self::new(Self::PROPFIND, uri, version)
    }

    pub fn report(uri: &str, version: &str) -> Self {
        Self::new(Self::REPORT, uri, version)
    }

    pub fn basic_auth(mut self, user: &str, pass: &str) -> Self {
        let auth = BASE64_STANDARD.encode(format!("{user}:{pass}"));
        self.bytes.extend(b"Authorization: Basic ");
        self.bytes.extend(auth.as_bytes());
        self.bytes.extend(b"\r\n");
        self
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.bytes.extend(key.as_bytes());
        self.bytes.extend(b": ");
        self.bytes.extend(value.as_bytes());
        self.bytes.extend(b"\r\n");
        self
    }

    pub fn depth(self, value: &str) -> Self {
        self.header("Depth", value)
    }

    pub fn connection(self, value: &str) -> Self {
        self.header("Connection", value)
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
