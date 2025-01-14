use base64::{prelude::BASE64_STANDARD, Engine};
use http::{
    header::{AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE, DATE},
    Request,
};
use httparse::{Header, Response, Status, EMPTY_HEADER};

use crate::serde::{AddressDataProp, Multistatus};

use super::{EnqueueResponseBytes, Flow, Io, TakeRequestBytes};

const REPORT: &str = "REPORT";
const DEPTH: &str = "DEPTH";

#[derive(Clone, Debug)]
pub enum State {
    SerializeHttpRequest,
    SendHttpRequest,
    ReceiveHttpResponseHeader,
    ReceiveHttpResponseBody,
    DeserializeHttpResponse,
}

/// [`Flow`] for listing a secret from a keyring contacts.
#[derive(Debug)]
pub struct ListContactsFlow {
    host: String,
    port: u16,
    state: Option<State>,

    read_bytes: Vec<u8>,
    read_bytes_count: usize,

    write_buf: Vec<u8>,

    response_bytes: Vec<u8>,
    response_body_length: usize,
    response_headers: [Header<'static>; 16],

    contacts: Option<Result<Multistatus<AddressDataProp>, quick_xml::de::DeError>>,
}

impl ListContactsFlow {
    /// Creates a new [`ListContactsFlow`] from the given keyring contacts
    /// key.
    pub fn new(host: impl ToString, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            state: Some(State::SerializeHttpRequest),
            read_bytes: vec![0; 16],
            read_bytes_count: 0,
            write_buf: vec![],
            response_bytes: vec![],
            response_body_length: 0,
            response_headers: [EMPTY_HEADER; 16],
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
            match self.state.take() {
                None => return None,
                Some(State::SerializeHttpRequest) => {
                    let auth = BASE64_STANDARD.encode(format!("test:test"));

                    let uri = format!(
                        "http://{}:{}/test/6fa928d4-e344-3021-1ad2-c652209ae251/",
                        self.host, self.port
                    );

                    let body = br#"
                    <C:addressbook-query xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:carddav">
                        <D:prop>
                            <D:getetag />
                            <D:getlastmodified />
                            <C:address-data />
                        </D:prop>
                    </C:addressbook-query>
                "#;

                    let (parts, body) = Request::builder()
                        .method(REPORT)
                        .uri(uri)
                        .header(DATE, "Mon, 13 Jan 2025 14:44:31 GMT")
                        .header(CONTENT_TYPE, "text/plain; charset=utf-8")
                        .header(CONTENT_LENGTH, body.len())
                        .header(AUTHORIZATION, auth)
                        .header(DEPTH, 0)
                        .body(body)
                        .unwrap()
                        .into_parts();

                    let mut request_bytes = Vec::<u8>::new();

                    request_bytes.extend(parts.method.as_str().as_bytes());
                    request_bytes.push(b' ');

                    request_bytes.extend(parts.uri.path_and_query().unwrap().as_str().as_bytes());
                    request_bytes.push(b' ');

                    request_bytes.extend(format!("{:?}", parts.version).as_bytes());
                    request_bytes.push(b'\r');
                    request_bytes.push(b'\n');

                    for (k, v) in parts.headers {
                        if let Some(k) = k {
                            request_bytes.extend(k.as_str().as_bytes());
                            request_bytes.push(b':');
                            request_bytes.push(b' ');
                            request_bytes.extend(v.as_bytes());
                            request_bytes.push(b'\r');
                            request_bytes.push(b'\n');
                        }
                    }

                    request_bytes.push(b'\r');
                    request_bytes.push(b'\n');

                    request_bytes.extend(body);

                    self.write_buf = request_bytes;
                    self.state = Some(State::SendHttpRequest);
                    return Some(Io::TcpWrite);
                }
                Some(State::SendHttpRequest) => {
                    self.state = Some(State::ReceiveHttpResponseHeader);
                    return Some(Io::TcpRead);
                }
                Some(State::ReceiveHttpResponseHeader) => {
                    let bytes = &self.read_bytes[..self.read_bytes_count];
                    println!(
                        "bytes({}): {:?}",
                        self.read_bytes_count,
                        String::from_utf8_lossy(bytes)
                    );

                    if self.response_body_length == 0 {
                        for header in self.response_headers {
                            if header.name.eq_ignore_ascii_case(CONTENT_LENGTH.as_str()) {
                                // FIXME: find a better way?
                                self.response_body_length = String::from_utf8_lossy(header.value)
                                    .parse::<usize>()
                                    .unwrap();
                                break;
                            }
                        }
                    }

                    match self.response.as_mut().unwrap().parse(&bytes).unwrap() {
                        Status::Partial => {
                            self.state = Some(State::ReceiveHttpResponseHeader);
                            return Some(Io::TcpRead);
                        }
                        Status::Complete(n) => {
                            let body = &bytes[n..];
                            self.response_body_length -= body.len();
                            self.response_bytes.extend(body);

                            if self.response_body_length == 0 {
                                self.state = Some(State::DeserializeHttpResponse);
                                continue;
                            } else {
                                self.state = Some(State::ReceiveHttpResponseBody);
                                return Some(Io::TcpRead);
                            }
                        }
                    }
                }
                Some(State::ReceiveHttpResponseBody) => {
                    let bytes = &self.read_bytes[..self.read_bytes_count];
                    self.response_body_length -= bytes.len();
                    self.response_bytes.extend(bytes);

                    if self.response_body_length == 0 {
                        self.state = Some(State::DeserializeHttpResponse);
                        continue;
                    } else {
                        self.state = Some(State::ReceiveHttpResponseBody);
                        return Some(Io::TcpRead);
                    }
                }
                Some(State::DeserializeHttpResponse) => {
                    let bytes = self.response_bytes.as_slice();
                    self.contacts = Some(quick_xml::de::from_reader(bytes));
                    return None;
                }
            }
        }
    }
}
