use quick_xml::DeError as Error;
use uuid::Uuid;

use crate::{
    carddav::serde::{AddressbookProp, MkcolResponse},
    http::sans_io::{Request, SendReceiveFlow},
    tcp::sans_io::{Flow, Io, Read, Write},
};

#[derive(Debug)]
pub struct CreateAddressbookFlow {
    http: SendReceiveFlow,
}

impl CreateAddressbookFlow {
    pub fn new(
        uri: impl AsRef<str>,
        version: impl AsRef<str>,
        user: impl AsRef<str>,
        pass: impl AsRef<str>,
        name: impl AsRef<str>,
        desc: Option<impl AsRef<str>>,
        color: Option<impl AsRef<str>>,
    ) -> Self {
        let name = name.as_ref();
        let uuid = Uuid::new_v4();
        let uri = &format!("{}/{uuid:x}", uri.as_ref());

        let color = match color {
            Some(color) => format!(
                "<I:addressbook-color>{}</I:addressbook-color>",
                color.as_ref()
            ),
            None => String::new(),
        };

        let desc = match desc {
            Some(desc) => format!(
                "<C:addressbook-description>{}</C:addressbook-description>",
                desc.as_ref()
            ),
            None => String::new(),
        };

        let request = Request::mkcol(uri, version.as_ref())
            .content_type_xml()
            .basic_auth(user.as_ref(), pass.as_ref())
            .body(&format!(
                include_str!("./flow-addressbook-create.xml"),
                name, color, desc,
            ));

        Self {
            http: SendReceiveFlow::new(request),
        }
    }

    pub fn output(self) -> Result<MkcolResponse<AddressbookProp>, Error> {
        quick_xml::de::from_reader(self.http.take_body().as_slice())
    }
}

impl Flow for CreateAddressbookFlow {}

impl Write for CreateAddressbookFlow {
    fn get_buffer(&mut self) -> &[u8] {
        self.http.get_buffer()
    }

    fn set_wrote_bytes_count(&mut self, count: usize) {
        self.http.set_wrote_bytes_count(count)
    }
}

impl Read for CreateAddressbookFlow {
    fn get_buffer_mut(&mut self) -> &mut [u8] {
        self.http.get_buffer_mut()
    }

    fn set_read_bytes_count(&mut self, count: usize) {
        self.http.set_read_bytes_count(count)
    }
}

impl Iterator for CreateAddressbookFlow {
    type Item = Io;

    fn next(&mut self) -> Option<Self::Item> {
        self.http.next()
    }
}
