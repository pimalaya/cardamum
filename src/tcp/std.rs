use std::{
    io::{Read, Write},
    net::TcpStream,
};

use thiserror::Error;

use super::sans_io::{ReadBytes, WriteBytes};

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct StdConnector {
    stream: TcpStream,
}

impl StdConnector {
    pub fn connect(host: impl AsRef<str>, port: u16) -> Result<Self> {
        let stream = TcpStream::connect((host.as_ref(), port))?;
        Ok(Self { stream })
    }

    pub fn read<F: ReadBytes>(&mut self, flow: &mut F) -> Result<()> {
        let buffer = flow.get_buffer_mut();
        let count = self.stream.read(buffer)?;
        flow.set_read_bytes_count(count);
        Ok(())
    }

    pub fn write<F: WriteBytes>(&mut self, flow: &mut F) -> Result<()> {
        let buffer = flow.get_buffer();
        let count = self.stream.write(buffer)?;
        flow.set_wrote_bytes_count(count);
        Ok(())
    }
}
