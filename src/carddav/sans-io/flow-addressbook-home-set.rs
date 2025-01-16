use crate::{
    carddav::serde::{AddressbookHomeSetProp, Multistatus},
    http::sans_io::{Request, SendReceiveFlow},
    tcp::sans_io::{Flow, Io, Read, Write},
};

#[derive(Debug)]
pub struct AddressbookHomeSetFlow {
    http: SendReceiveFlow<Multistatus<AddressbookHomeSetProp>>,
}

impl AddressbookHomeSetFlow {
    const BODY: &str = r#"
        <propfind xmlns="DAV:" xmlns:C="urn:ietf:params:xml:ns:carddav">
            <prop>
                <C:addressbook-home-set />
            </prop>
        </propfind>
    "#;

    pub fn new(
        uri: impl AsRef<str>,
        version: impl AsRef<str>,
        user: impl AsRef<str>,
        pass: impl AsRef<str>,
    ) -> Self {
        let request = Request::propfind(uri.as_ref(), version.as_ref())
            .basic_auth(user.as_ref(), pass.as_ref())
            .body(Self::BODY);

        Self {
            http: SendReceiveFlow::new(request),
        }
    }

    pub fn output(
        self,
    ) -> Option<Result<Multistatus<AddressbookHomeSetProp>, quick_xml::de::DeError>> {
        self.http.output()
    }
}

impl Flow for AddressbookHomeSetFlow {}

impl Write for AddressbookHomeSetFlow {
    fn get_buffer(&mut self) -> &[u8] {
        self.http.get_buffer()
    }

    fn set_wrote_bytes_count(&mut self, count: usize) {
        self.http.set_wrote_bytes_count(count)
    }
}

impl Read for AddressbookHomeSetFlow {
    fn get_buffer_mut(&mut self) -> &mut [u8] {
        self.http.get_buffer_mut()
    }

    fn set_read_bytes_count(&mut self, count: usize) {
        self.http.set_read_bytes_count(count)
    }
}

impl Iterator for AddressbookHomeSetFlow {
    type Item = Io;

    fn next(&mut self) -> Option<Self::Item> {
        self.http.next()
    }
}
