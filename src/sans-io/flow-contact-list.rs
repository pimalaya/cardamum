use base64::{prelude::BASE64_STANDARD, Engine};
use chrono::Utc;
use memchr::memmem;

use crate::serde::{AddressDataProp, Multistatus};

use super::{EnqueueResponseBytes, Flow, Io, TakeRequestBytes};

const LF: u8 = b'\n';
const CR: u8 = b'\r';

const CONTENT_LENGTH: &[u8] = b"\r\nContent-Length";

#[derive(Clone, Debug)]
pub enum State {
    SerializeHttpRequest,
    SendHttpRequest,
    ReceiveHttpResponse,
    DeserializeHttpResponse,
}

/// [`Flow`] for listing a secret from a keyring contacts.
#[derive(Debug)]
pub struct ListContactsFlow {
    user: String,
    collection_id: String,

    state: Option<State>,

    read_bytes: Vec<u8>,
    read_bytes_count: usize,

    write_buf: Vec<u8>,

    response_bytes: Vec<u8>,
    response_body_start: usize,
    response_body_length: usize,

    contacts: Option<Result<Multistatus<AddressDataProp>, quick_xml::de::DeError>>,
}

impl ListContactsFlow {
    /// Creates a new [`ListContactsFlow`] from the given keyring contacts
    /// key.
    pub fn new(user: impl ToString, collection_id: impl ToString) -> Self {
        Self {
            user: user.to_string(),
            collection_id: collection_id.to_string(),
            state: Some(State::SerializeHttpRequest),
            read_bytes: vec![0; 16],
            read_bytes_count: 0,
            write_buf: vec![],
            response_bytes: vec![],
            response_body_start: 0,
            response_body_length: 0,
            contacts: None,
        }
    }

    pub fn take_contacts(
        &mut self,
    ) -> Option<Result<Multistatus<AddressDataProp>, quick_xml::de::DeError>> {
        self.contacts.take()
    }
}

impl Flow for ListContactsFlow {}

impl TakeRequestBytes for ListContactsFlow {
    fn take_request_bytes(&mut self) -> Vec<u8> {
        self.write_buf.drain(..).collect()
    }
}

impl EnqueueResponseBytes for ListContactsFlow {
    fn buf(&mut self) -> &mut [u8] {
        &mut self.read_bytes
    }

    fn read_bytes_count(&mut self, count: usize) {
        self.read_bytes_count = count;
    }
}

impl Iterator for ListContactsFlow {
    type Item = Io;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let state = self.state.take();
            println!("state: {state:?}");

            match state {
                None => return None,
                Some(State::SerializeHttpRequest) => {
                    let body = r#"
                        <C:addressbook-query xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:carddav">
                            <D:prop>
                                <D:getetag />
                                <D:getlastmodified />
                                <C:address-data />
                            </D:prop>
                        </C:addressbook-query>
                    "#;

                    let uri = format!("/{}/{}", self.user, self.collection_id);

                    let request = Request::report(&uri)
                        .basic_auth("test", "test")
                        .depth("1")
                        .body(body);

                    self.state = Some(State::SendHttpRequest);
                    self.write_buf = request.into();
                    println!("request: {:?}", String::from_utf8_lossy(&self.write_buf));
                    return Some(Io::TcpWrite);
                }
                Some(State::SendHttpRequest) => {
                    self.state = Some(State::ReceiveHttpResponse);
                    return Some(Io::TcpRead);
                }
                Some(State::ReceiveHttpResponse) => {
                    let bytes = &self.read_bytes[..self.read_bytes_count];
                    self.response_bytes.extend(bytes);

                    println!(
                        "bytes({}/{}): {:?}",
                        self.read_bytes_count,
                        self.read_bytes.len(),
                        String::from_utf8_lossy(bytes)
                    );

                    if self.response_body_start == 0 {
                        let body_start = memmem::find(&self.response_bytes, &[CR, LF, CR, LF]);

                        if let Some(n) = body_start {
                            self.response_body_start = n + 4;
                        }
                    }

                    if self.response_body_start > 0 && self.response_body_length == 0 {
                        let content_length = memmem::find(&self.response_bytes, CONTENT_LENGTH);

                        if let Some(mut begin) = content_length {
                            begin += CONTENT_LENGTH.len() + 1;

                            let bytes = &self.response_bytes[begin..];
                            let end = memmem::find(bytes, &[CR, LF]).unwrap();

                            let content_length = &bytes[..end];
                            let content_length = String::from_utf8_lossy(content_length);
                            self.response_body_length = content_length.trim().parse().unwrap();
                        }
                    }

                    if self.response_body_start > 0 && self.response_body_length > 0 {
                        let body_bytes = &self.response_bytes[self.response_body_start..];
                        if body_bytes.len() >= self.response_body_length {
                            self.state = Some(State::DeserializeHttpResponse);
                            continue;
                        }
                    }

                    self.state = Some(State::ReceiveHttpResponse);
                    return Some(Io::TcpRead);
                }
                Some(State::DeserializeHttpResponse) => {
                    let bytes = &self.response_bytes[self.response_body_start..];
                    self.contacts = Some(quick_xml::de::from_reader(bytes));
                    return None;
                }
            }
        }
    }
}

pub struct Request {
    bytes: Vec<u8>,
}

impl Request {
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

        bytes.extend(b"Content-Type: application/xml\r\n");

        Self { bytes }
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
