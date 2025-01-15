use std::mem;

use memchr::memmem;
use serde::Deserialize;

use crate::tcp::sans_io::{Flow, Io, ReadBytes, WriteBytes};

use super::{Request, State};

const LF: u8 = b'\n';
const CR: u8 = b'\r';

const CONTENT_LENGTH: &[u8] = b"\r\nContent-Length";

#[derive(Debug)]
pub struct SendReceiveFlow<T> {
    state: Option<State>,

    write_buffer: Vec<u8>,
    wrote_bytes_count: usize,

    read_buffer: Vec<u8>,
    read_bytes_count: usize,

    request: Request,
    response_bytes: Vec<u8>,
    response_body_start: usize,
    response_body_length: usize,

    output: Option<Result<T, quick_xml::de::DeError>>,
}

impl<T> SendReceiveFlow<T> {
    pub fn new(request: Request) -> Self {
        Self {
            state: Some(State::SerializeHttpRequest),

            write_buffer: vec![],
            wrote_bytes_count: 0,

            read_buffer: vec![0; 512],
            read_bytes_count: 0,

            request,
            response_bytes: vec![],
            response_body_start: 0,
            response_body_length: 0,

            output: None,
        }
    }

    pub fn output(mut self) -> Option<Result<T, quick_xml::de::DeError>> {
        self.output.take()
    }
}

impl<T: for<'de> Deserialize<'de>> Flow for SendReceiveFlow<T> {}

impl<T: for<'de> Deserialize<'de>> WriteBytes for SendReceiveFlow<T> {
    fn get_buffer(&mut self) -> &[u8] {
        &self.write_buffer
    }

    fn set_wrote_bytes_count(&mut self, count: usize) {
        self.wrote_bytes_count = count;
    }
}

impl<T: for<'de> Deserialize<'de>> ReadBytes for SendReceiveFlow<T> {
    fn get_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.read_buffer
    }

    fn set_read_bytes_count(&mut self, count: usize) {
        self.read_bytes_count = count;
    }
}

impl<T: for<'de> Deserialize<'de>> Iterator for SendReceiveFlow<T> {
    type Item = Io;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.state.take() {
                None => return None,
                Some(State::SerializeHttpRequest) => {
                    self.state = Some(State::SendHttpRequest);
                    let mut request = Request::default();
                    mem::swap(&mut request, &mut self.request);
                    self.write_buffer = request.into();
                    return Some(Io::Write);
                }
                Some(State::SendHttpRequest) => {
                    self.state = Some(State::ReceiveHttpResponse);
                    return Some(Io::Read);
                }
                Some(State::ReceiveHttpResponse) => {
                    let bytes = &self.read_buffer[..self.read_bytes_count];
                    self.response_bytes.extend(bytes);

                    // println!(
                    //     "bytes({}/{}): {:?}",
                    //     self.read_bytes_count,
                    //     self.read_buffer.len(),
                    //     String::from_utf8_lossy(bytes)
                    // );

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
                    return Some(Io::Read);
                }
                Some(State::DeserializeHttpResponse) => {
                    let bytes = &self.response_bytes[self.response_body_start..];
                    self.output = Some(quick_xml::de::from_reader(bytes));
                    return None;
                }
            }
        }
    }
}
