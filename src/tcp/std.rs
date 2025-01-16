use std::{
    io::{Read, Result, Write},
    net::TcpStream,
};

use super::sans_io::{Read as TcpRead, Write as TcpWrite};

#[derive(Debug)]
pub struct Connector {
    stream: TcpStream,
}

impl Connector {
    pub fn connect(host: impl AsRef<str>, port: u16) -> Result<Self> {
        let stream = TcpStream::connect((host.as_ref(), port))?;
        Ok(Self { stream })
    }

    pub fn read<F: TcpRead>(&mut self, flow: &mut F) -> Result<()> {
        let buffer = flow.get_buffer_mut();
        let read_bytes_count = self.stream.read(buffer)?;
        flow.set_read_bytes_count(read_bytes_count);
        Ok(())
    }

    pub fn write<F: TcpWrite>(&mut self, flow: &mut F) -> Result<()> {
        let buffer = flow.get_buffer();
        let wrote_bytes_count = self.stream.write(buffer)?;
        flow.set_wrote_bytes_count(wrote_bytes_count);
        Ok(())
    }
}
