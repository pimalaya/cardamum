//! # GET command
//!
//! Fetches the raw vCard bytes of one card with a WebDAV `GET`.

use core::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{output::Bytes, printer::Printer};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    carddav::client::{CarddavClient, card_refusal},
    shared::card::read::{CardContents, CardEncoding},
};

/// GET a card: its raw vCard bytes plus the server ETag.
///
/// JSON output: `{"id", "etag", "contents", "encoding"}`, `contents` the
/// vCard text (`encoding` `utf-8`) or the base64 of bytes that are not
/// UTF-8 (`encoding` `base64`).
#[derive(Debug, Parser)]
pub struct CarddavGetCommand {
    /// Identifier of the parent addressbook.
    #[arg(value_name = "ADDRESSBOOK")]
    pub addressbook_id: String,
    /// Card resource id (its href last path segment).
    #[arg(value_name = "CARD-ID")]
    pub card_id: String,
}

impl CarddavGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CarddavClient) -> Result<()> {
        let body = client
            .read_card(&self.addressbook_id, &self.card_id)
            .map_err(card_refusal)?;

        let contents = CardContents::new(body.data);

        // NOTE: a card that is not UTF-8 goes out as its bytes in text,
        // as `card build` hands one over.
        if contents.encoding == CardEncoding::Base64 && !printer.is_json() {
            let mime = String::from("text/vcard");
            return printer.out_bytes(Bytes {
                data: contents.bytes,
                mime,
            });
        }

        printer.out(CarddavGetOutput {
            id: self.card_id,
            etag: body.etag,
            contents,
        })
    }
}

/// A fetched card: its raw vCard bytes and the server ETag.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CarddavGetOutput {
    /// Card resource id, its href last path segment.
    pub id: String,
    /// The server ETag, when it sent one.
    pub etag: Option<String>,
    /// The raw vCard, as text or as base64.
    #[serde(flatten)]
    pub contents: CardContents,
}

impl fmt::Display for CarddavGetOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.contents.bytes))
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use io_webdav::{client::WebdavClientStd, rfc4918::WebdavAuth};
    use pimalaya_cli::printer::CapturePrinter;
    use serde_json::Value;
    use url::Url;

    use super::CarddavGetCommand;
    use crate::{carddav::client::CarddavClient, script::Script};

    /// The JSON `carddav get book c1.vcf` prints over a server answering
    /// the card `body`.
    fn get(body: &[u8]) -> Value {
        let head = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/vcard\r\nETag: \"e1\"\r\n\
             Content-Length: {}\r\n\r\n",
            body.len()
        );
        let (script, _) = Script::new([[head.as_bytes(), body].concat()]);
        let base = Url::parse("https://dav.example.org/").unwrap();
        let home = base.join("/dav/").unwrap();
        let inner =
            WebdavClientStd::from_parts(script, WebdavAuth::None, base, None, None, Some(home));

        let cmd = CarddavGetCommand::try_parse_from(["get", "book", "c1.vcf"]).unwrap();
        let mut printer = CapturePrinter::default();
        cmd.execute(&mut printer, CarddavClient::new(inner))
            .unwrap();
        printer.into_single().unwrap()
    }

    /// The vCard travels beside `encoding`, as every vCard string does
    /// (pimalaya-cli output.md, "Bytes are never raw under JSON").
    #[test]
    fn a_card_reads_beside_its_encoding() {
        let card = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Ada\r\nEND:VCARD\r\n";
        let json = get(card.as_bytes());

        assert_eq!(json["id"], "c1.vcf", "{json}");
        assert_eq!(json["etag"], "e1", "{json}");
        assert_eq!(json["contents"], card, "{json}");
        assert_eq!(json["encoding"], "utf-8", "{json}");
    }

    /// A card whose bytes are not UTF-8 reads as their base64, where the
    /// command failed.
    #[test]
    fn a_card_that_is_not_utf8_reads_as_base64() {
        let json = get(b"BEGIN:VCARD\r\nFN:Ad\xe9\r\nEND:VCARD\r\n");

        let encoded = "QkVHSU46VkNBUkQNCkZOOkFk6Q0KRU5EOlZDQVJEDQo=";
        assert_eq!(json["contents"], encoded, "{json}");
        assert_eq!(json["encoding"], "base64", "{json}");
    }
}
