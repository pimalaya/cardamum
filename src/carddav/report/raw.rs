//! # Raw report command
//!
//! Sends an arbitrary XML REPORT body to an addressbook and renders the
//! multistatus it answers.

use std::{
    fmt, fs,
    io::{Read, Write, stdin},
    path::PathBuf,
};

use anyhow::{Context, Result, anyhow, bail};
use clap::Parser;
use io_webdav::{
    coroutine::{WebdavCoroutine, WebdavCoroutineState, WebdavYield},
    rfc4918::{GETETAG, WebdavMultistatus, collection_path, report::WebdavReport},
    rfc6352::addressbook::ADDRESS_DATA,
};
use pimalaya_cli::{columns::Columns, printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::carddav::client::{CarddavClient, check_addressbook_id};

/// Run a REPORT with an arbitrary XML body against an addressbook.
///
/// The escape hatch for the `addressbook-query`,
/// `addressbook-multiget` and `sync-collection` variants the typed
/// subcommands do not cover. The XML body comes from a file, inline, or
/// `-` for stdin, and the parsed multistatus is printed.
///
/// JSON output: `{"responses": [{"href", "status", "etag",
/// "dataBytes"}], "syncToken"}`.
#[derive(Debug, Parser)]
pub struct CarddavReportRawCommand {
    /// Identifier of the addressbook to run the REPORT against.
    #[arg(value_name = "ADDRESSBOOK")]
    pub addressbook_id: String,
    /// XML REPORT body: a file path, raw XML, or `-` for stdin.
    #[arg(value_name = "XML")]
    pub xml: String,
    /// `Depth` header (0 or 1).
    #[arg(long, default_value_t = 1)]
    pub depth: u8,
}

impl CarddavReportRawCommand {
    /// Runs the REPORT on the address book's path, built as io-webdav
    /// builds it for its own requests: an id addressing another collection
    /// is refused first, a relative one percent-encoded.
    pub fn execute(self, printer: &mut impl Printer, mut client: CarddavClient) -> Result<()> {
        check_addressbook_id(&self.addressbook_id)?;
        let body = read_xml(&self.xml)?;

        let home = client
            .addressbook_home_set
            .clone()
            .ok_or_else(|| anyhow!("Addressbook home-set is not resolved"))?;
        let path = collection_path(home.path(), &self.addressbook_id);
        let base_url = client.base_url.clone();
        let user_agent = client.user_agent.clone();
        let auth = client.auth().clone();

        let coroutine = WebdavReport::new(&base_url, &auth, &user_agent, &path, self.depth, body);
        let multistatus = run_report(&mut client, coroutine)?;

        printer.out(CarddavReportRawOutput {
            sync_token: multistatus.sync_token,
            responses: multistatus.responses.iter().map(EntryRow::from).collect(),
        })
    }
}

/// Reads the XML body from stdin (`-`), a file, or the value itself.
///
/// An existing path is read from disk, otherwise a value starting with
/// `<` is taken as inline XML.
fn read_xml(source: &str) -> Result<Vec<u8>> {
    if source == "-" {
        pimalaya_cli::session::check_stdin(crate::error::ErrorCode::StdinTaken)?;
        let mut buf = Vec::new();
        stdin()
            .read_to_end(&mut buf)
            .context("Read REPORT body from stdin error")?;
        return Ok(buf);
    }

    let path = PathBuf::from(source);
    if path.is_file() {
        return fs::read(&path)
            .with_context(|| format!("Read REPORT body from `{}` error", path.display()));
    }

    if source.trim_start().starts_with('<') {
        return Ok(source.as_bytes().to_vec());
    }

    bail!("Source `{source}` is neither a readable file nor XML");
}

/// Pumps a [`WebdavReport`] coroutine over the client's stream.
fn run_report(
    client: &mut CarddavClient,
    mut coroutine: WebdavReport,
) -> Result<WebdavMultistatus> {
    let mut buf = [0u8; 8 * 1024];
    let mut arg: Option<&[u8]> = None;

    loop {
        match coroutine.resume(arg.take()) {
            WebdavCoroutineState::Complete(result) => return Ok(result?),
            WebdavCoroutineState::Yielded(WebdavYield::WantsWrite(bytes)) => {
                client.stream.write_all(&bytes)?;
            }
            WebdavCoroutineState::Yielded(WebdavYield::WantsRead) => {
                let n = client.stream.read(&mut buf)?;
                arg = Some(&buf[..n]);
            }
        }
    }
}

/// Multistatus a raw REPORT answered.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CarddavReportRawOutput {
    /// One entry per multistatus response.
    pub responses: Vec<EntryRow>,
    /// The `sync-token` the multistatus carried, when it carried one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_token: Option<String>,
}

/// One multistatus response: href, status, ETag and body size.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EntryRow {
    /// The response href.
    pub href: String,
    /// Its status code, when the response carried one.
    pub status: Option<u16>,
    /// Its `getetag` property, unquoted.
    pub etag: Option<String>,
    /// Size of its `address-data` property, in bytes: the whole card, its
    /// final line break included.
    pub data_bytes: usize,
}

impl From<&io_webdav::rfc4918::WebdavResponseEntry> for EntryRow {
    fn from(entry: &io_webdav::rfc4918::WebdavResponseEntry) -> Self {
        Self {
            href: entry.href.clone(),
            status: entry.status,
            etag: entry
                .text(GETETAG)
                .map(|raw| raw.trim_matches('"').to_string()),
            // NOTE: `text` trims the final CRLF; the property keeps it whole.
            data_bytes: entry
                .prop(ADDRESS_DATA)
                .map(|prop| prop.text.len())
                .unwrap_or(0),
        }
    }
}

impl fmt::Display for CarddavReportRawOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.responses.is_empty() {
            let mut columns = Columns::new(["STATUS", "DATA-BYTES", "ETAG", "HREF"]);

            for entry in &self.responses {
                let status = entry.status.map(|s| s.to_string()).unwrap_or_default();
                let etag = entry.etag.as_deref().map(sanitize::text);

                columns.push([
                    status,
                    entry.data_bytes.to_string(),
                    etag.map(|etag| etag.into_owned()).unwrap_or_default(),
                    sanitize::text(&entry.href).into_owned(),
                ]);
            }

            write!(f, "{columns}")?;
        }

        if let Some(token) = &self.sync_token {
            if !self.responses.is_empty() {
                writeln!(f)?;
            }

            writeln!(f, "sync-token: {}", sanitize::text(token))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use io_webdav::{client::WebdavClientStd, rfc4918::WebdavAuth};
    use pimalaya_cli::{error::code_of, printer::CapturePrinter};
    use url::Url;

    use super::CarddavReportRawCommand;
    use crate::{
        carddav::client::CarddavClient,
        error::ErrorCode,
        script::{self, Requests, Script},
    };

    /// The card the server answers, each line ending in a CRLF.
    const CARD: &str = "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:c1\r\nFN:Jane\r\nEND:VCARD\r\n";

    /// A multistatus answering [`CARD`], each CR escaped as iCloud and
    /// Fastmail escape it.
    const MULTISTATUS: &str = r#"<?xml version="1.0"?>
<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:carddav">
  <d:response>
    <d:href>/dav/My%20Book/c1.vcf</d:href>
    <d:propstat>
      <d:prop>
        <d:getetag>"e1"</d:getetag>
        <c:address-data>BEGIN:VCARD&#13;
VERSION:4.0&#13;
UID:c1&#13;
FN:Jane&#13;
END:VCARD&#13;
</c:address-data>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
</d:multistatus>"#;

    /// A client whose home set is `/dav/`, over a server answering
    /// [`MULTISTATUS`], and the requests it receives.
    fn client() -> (CarddavClient, Requests) {
        let response = script::response(207, "application/xml", MULTISTATUS);
        let (script, requests) = Script::new([response]);
        let base = Url::parse("https://dav.example.org/").unwrap();
        let home = base.join("/dav/").unwrap();
        let inner =
            WebdavClientStd::from_parts(script, WebdavAuth::None, base, None, None, Some(home));
        (CarddavClient::new(inner), requests)
    }

    /// The command running a REPORT on `addressbook_id`.
    fn report(addressbook_id: &str) -> CarddavReportRawCommand {
        CarddavReportRawCommand {
            addressbook_id: addressbook_id.to_owned(),
            xml: String::from("<x/>"),
            depth: 1,
        }
    }

    /// The size printed is the whole card's, its final CRLF included, and
    /// the path the one io-webdav builds, percent-encoded.
    #[test]
    fn a_raw_report_counts_the_whole_card_on_an_encoded_path() {
        let (client, requests) = client();
        let mut printer = CapturePrinter::default();
        report("My Book").execute(&mut printer, client).unwrap();

        let output = printer.into_single().unwrap();
        assert_eq!(output["responses"][0]["dataBytes"], CARD.len(), "{output}");

        let requests = requests.lock().unwrap();
        let request = String::from_utf8_lossy(&requests[0]);
        assert!(
            request.starts_with("REPORT /dav/My%20Book/ HTTP/1.1\r\n"),
            "{request}"
        );
    }

    /// An id addressing the home set, its parent or the server root is
    /// refused `addressbook-not-found`, nothing sent.
    #[test]
    fn a_raw_report_refuses_an_id_out_of_its_home() {
        for id in ["..", "", "/", ".", "a/../..", "%2e%2e", "a//b"] {
            let (client, requests) = client();
            let mut printer = CapturePrinter::default();
            let err = report(id).execute(&mut printer, client).unwrap_err();

            assert_eq!(
                code_of(&err),
                Some(ErrorCode::AddressbookNotFound),
                "{id:?}"
            );
            assert_eq!(err.to_string(), format!("Addressbook `{id}` not found"));
            assert!(requests.lock().unwrap().is_empty(), "{id:?}");
        }
    }
}
