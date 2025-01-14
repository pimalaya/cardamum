/// Trait used for building keyring-related sans I/O state machine
/// flows.
///
/// A flow is defined as an iterable state machine, where every
/// `.next()` call produces a potential [`Io`] that needs to be
/// performed outside of the flow, and makes the state go forward. No
/// [`Io`] produced means that the flow is terminated and does not
/// require any longer [`Io`] to be performed.
pub trait Flow: Iterator {}

/// Trait dedicated to flows that needs to take secrets.
///
/// This trait make sure that the given flow knows how to take a request
/// into its inner state.
pub trait TakeRequestBytes: Flow {
    fn take_request_bytes(&mut self) -> Vec<u8>;
}

/// Trait dedicated to flows that needs to put secrets.
///
/// This trait make sure that the given flow knows how to put a response
/// into its inner state.
pub trait EnqueueResponseBytes: Flow {
    fn buf(&mut self) -> &mut [u8];
    fn read_bytes_count(&mut self, count: usize);
}
