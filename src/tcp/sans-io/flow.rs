pub trait Flow: Iterator {}

pub trait ReadBytes: Flow {
    fn get_buffer_mut(&mut self) -> &mut [u8];
    fn set_read_bytes_count(&mut self, count: usize);
}

pub trait WriteBytes: Flow {
    fn get_buffer(&mut self) -> &[u8];
    fn set_wrote_bytes_count(&mut self, count: usize);
}
