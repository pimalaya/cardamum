#[derive(Clone, Debug)]
pub enum State {
    SerializeHttpRequest,
    SendHttpRequest,
    ReceiveHttpResponse,
    DeserializeHttpResponse,
}
