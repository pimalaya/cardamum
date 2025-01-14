/// The type of I/O generated by keyring [`Flow`] state machines.
///
/// This enum is the representation of I/Os that need to be performed
/// outside of [`Flow`]s.
#[derive(Clone, Debug)]
pub enum Io {
    TcpRead,
    TcpWrite,
}
