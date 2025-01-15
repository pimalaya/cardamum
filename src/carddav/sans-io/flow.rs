pub trait Flow: Iterator {}

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
