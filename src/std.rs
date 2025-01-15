use std::{
    io::{Read, Write},
    net::TcpStream,
};

use thiserror::Error;

use crate::{
    sans_io::{EnqueueResponseBytes, Io, ListContactsFlow, TakeRequestBytes},
    serde::{AddressDataProp, Multistatus},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot connect to TCP stream")]
    ConnectToTcpStreamError(#[source] std::io::Error),
    #[error("cannot write bytes into TCP stream")]
    WriteBytesIntoTcpStreamError(#[source] std::io::Error),

    #[error("cannot find contacts")]
    TakeContactsError,

    #[error("cannot build HTTP request")]
    BuildRequestError(#[source] http::Error),

    #[error("cannot parse contacts from HTTP response body")]
    ParseContactsError(#[source] quick_xml::de::DeError),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct IoConnector {
    host: String,
    port: u16,
    stream: TcpStream,
}

impl IoConnector {
    pub fn new(host: impl ToString, port: u16) -> Result<Self> {
        let host = host.to_string();

        let stream = match TcpStream::connect((host.as_str(), port)) {
            Ok(stream) => stream,
            Err(err) => return Err(Error::ConnectToTcpStreamError(err)),
        };

        Ok(Self { host, port, stream })
    }

    pub fn read<F: EnqueueResponseBytes>(&mut self, flow: &mut F) -> Result<()> {
        match self.stream.read(flow.buf()) {
            Ok(n) => flow.read_bytes_count(n),
            Err(err) => return Err(Error::WriteBytesIntoTcpStreamError(err)),
        };

        Ok(())
    }

    pub fn write<F: TakeRequestBytes>(&mut self, flow: &mut F) -> Result<()> {
        let bytes = flow.take_request_bytes();

        match self.stream.write(&bytes) {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::WriteBytesIntoTcpStreamError(err)),
        }
    }

    pub fn list_contacts(&mut self, collection_id: &str) -> Result<Multistatus<AddressDataProp>> {
        let mut flow = ListContactsFlow::new("test", collection_id);

        while let Some(io) = flow.next() {
            match io {
                Io::TcpRead => {
                    self.read(&mut flow)?;
                }
                Io::TcpWrite => {
                    self.write(&mut flow)?;
                }
            }
        }

        match flow.take_contacts() {
            None => Err(Error::TakeContactsError),
            Some(Ok(contacts)) => Ok(contacts),
            Some(Err(err)) => Err(Error::ParseContactsError(err)),
        }
    }
}
