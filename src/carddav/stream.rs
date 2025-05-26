use std::{
    io::{self, Read, Write},
    net::TcpStream,
};

pub enum Stream {
    Plain(TcpStream),
    #[cfg(feature = "rustls")]
    Rustls(rustls::StreamOwned<rustls::ClientConnection, TcpStream>),
    #[cfg(feature = "native-tls")]
    NativeTls(native_tls::TlsStream<TcpStream>),
}

impl Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Plain(stream) => stream.read(buf),
            #[cfg(feature = "rustls")]
            Self::Rustls(stream) => stream.read(buf),
            #[cfg(feature = "native-tls")]
            Self::NativeTls(stream) => stream.read(buf),
        }
    }
}

impl Write for Stream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Self::Plain(stream) => stream.write(buf),
            #[cfg(feature = "rustls")]
            Self::Rustls(stream) => stream.write(buf),
            #[cfg(feature = "native-tls")]
            Self::NativeTls(stream) => stream.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Plain(stream) => stream.flush(),
            #[cfg(feature = "rustls")]
            Self::Rustls(stream) => stream.flush(),
            #[cfg(feature = "native-tls")]
            Self::NativeTls(stream) => stream.flush(),
        }
    }
}
