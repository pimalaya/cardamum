use crate::{
    carddav::serde::{CurrentUserPrincipalProp, Multistatus},
    http::sans_io::{Request, SendReceiveFlow},
    tcp::sans_io::{Flow, Io, Read, Write},
};

#[derive(Debug)]
pub struct CurrentUserPrincipalFlow {
    http: SendReceiveFlow<Multistatus<CurrentUserPrincipalProp>>,
}

impl CurrentUserPrincipalFlow {
    const BODY: &str = r#"
        <propfind xmlns="DAV:">
            <prop>
                <current-user-principal />
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
    ) -> Option<Result<Multistatus<CurrentUserPrincipalProp>, quick_xml::de::DeError>> {
        self.http.output()
    }
}

impl Flow for CurrentUserPrincipalFlow {}

impl Write for CurrentUserPrincipalFlow {
    fn get_buffer(&mut self) -> &[u8] {
        self.http.get_buffer()
    }

    fn set_wrote_bytes_count(&mut self, count: usize) {
        self.http.set_wrote_bytes_count(count)
    }
}

impl Read for CurrentUserPrincipalFlow {
    fn get_buffer_mut(&mut self) -> &mut [u8] {
        self.http.get_buffer_mut()
    }

    fn set_read_bytes_count(&mut self, count: usize) {
        self.http.set_read_bytes_count(count)
    }
}

impl Iterator for CurrentUserPrincipalFlow {
    type Item = Io;

    fn next(&mut self) -> Option<Self::Item> {
        self.http.next()
    }
}
