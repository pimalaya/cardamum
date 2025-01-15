use crate::{
    carddav::serde::{AddressDataProp, Multistatus},
    http::sans_io::{Request, SendReceiveFlow},
    tcp::sans_io::{Flow, Io, ReadBytes, WriteBytes},
};

#[derive(Debug)]
pub struct ListContactsFlow {
    http: SendReceiveFlow<Multistatus<AddressDataProp>>,
}

impl ListContactsFlow {
    const BODY: &str = r#"
        <C:addressbook-query xmlns="DAV:" xmlns:C="urn:ietf:params:xml:ns:carddav">
            <prop>
                <getetag />
                <getlastmodified />
                <C:address-data />
            </prop>
        </C:addressbook-query>
    "#;

    pub fn new(user: impl AsRef<str>, collection_id: impl AsRef<str>) -> Self {
        let user = user.as_ref();
        let collection_id = collection_id.as_ref();

        let uri = format!("/{user}/{collection_id}/");
        let request = Request::report(&uri)
            .basic_auth(user, "test")
            .depth("1")
            .body(Self::BODY);

        Self {
            http: SendReceiveFlow::new(request),
        }
    }

    pub fn output(self) -> Option<Result<Multistatus<AddressDataProp>, quick_xml::de::DeError>> {
        self.http.output()
    }
}

impl Flow for ListContactsFlow {}

impl WriteBytes for ListContactsFlow {
    fn get_buffer(&mut self) -> &[u8] {
        self.http.get_buffer()
    }

    fn set_wrote_bytes_count(&mut self, count: usize) {
        self.http.set_wrote_bytes_count(count)
    }
}

impl ReadBytes for ListContactsFlow {
    fn get_buffer_mut(&mut self) -> &mut [u8] {
        self.http.get_buffer_mut()
    }

    fn set_read_bytes_count(&mut self, count: usize) {
        self.http.set_read_bytes_count(count)
    }
}

impl Iterator for ListContactsFlow {
    type Item = Io;

    fn next(&mut self) -> Option<Self::Item> {
        self.http.next()
    }
}
