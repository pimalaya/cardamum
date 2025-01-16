use std::{
    io::{Read, Result, Write},
    net::TcpStream,
    sync::Arc,
};

use rustls::{ClientConfig, ClientConnection, StreamOwned};
use rustls_platform_verifier::ConfigVerifierExt;

use crate::tcp::sans_io::{Read as TcpRead, Write as TcpWrite};

#[derive(Debug)]
pub struct RustlsConnector {
    stream: StreamOwned<ClientConnection, TcpStream>,
}

impl RustlsConnector {
    pub fn connect(host: impl AsRef<str>, port: u16) -> Result<Self> {
        let config = ClientConfig::with_platform_verifier();
        let server_name = host.as_ref().to_owned().try_into().unwrap();
        let conn = ClientConnection::new(Arc::new(config), server_name).unwrap();
        let sock = TcpStream::connect((host.as_ref(), port))?;
        let stream = StreamOwned::new(conn, sock);

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
