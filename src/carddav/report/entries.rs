//! # Card entries
//!
//! The card list the `query` and `multiget` REPORT commands print.

use std::fmt;

use io_webdav::rfc6352::card::CarddavCardEntry;
use pimalaya_cli::{columns::Columns, error::ItemErrors, sanitize};
use schemars::JsonSchema;
use serde::Serialize;
use vcard::{
    prop::{VcardPropKind, VcardPropName},
    tree::cst::VcardCst,
    value::VcardValue,
};

use crate::error::ItemError;

/// Cards returned by a `query` or `multiget` REPORT.
///
/// The text lists ids, ETags and formatted names, then the errors; the
/// raw vCard body rides in `contents` for `--json`.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CarddavCardEntriesOutput {
    /// The cards the REPORT returned.
    #[serde(rename = "cards")]
    pub rows: Vec<EntryRow>,
    /// Whether the server truncated the result, more cards matching than
    /// it returned.
    pub truncated: bool,
    /// The error of each card the server answered without its data, in
    /// the order met.
    pub errors: Vec<ItemError>,
}

/// One returned card: its id, ETag and raw vCard body.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EntryRow {
    /// Card resource id, the last segment of its href, percent-decoded when the
    /// decoding encodes back to it.
    pub id: String,
    /// The server ETag, when it sent one.
    pub etag: Option<String>,
    /// The raw vCard, as text.
    pub contents: String,
    /// The first `FN` of the vCard, for the text output only.
    #[serde(skip)]
    pub fn_value: Option<String>,
}

impl From<CarddavCardEntry> for EntryRow {
    fn from(entry: CarddavCardEntry) -> Self {
        Self {
            fn_value: formatted_name(&entry.data),
            id: entry.id,
            etag: entry.etag,
            contents: String::from_utf8_lossy(&entry.data).into_owned(),
        }
    }
}

impl fmt::Display for CarddavCardEntriesOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.rows.is_empty() {
            let mut columns = Columns::new(["ID", "ETAG", "FN"]);

            for entry in &self.rows {
                let text = |value: &Option<String>| match value {
                    Some(value) => sanitize::text(value).into_owned(),
                    None => String::new(),
                };

                columns.push([
                    sanitize::text(&entry.id).into_owned(),
                    text(&entry.etag),
                    text(&entry.fn_value),
                ]);
            }

            write!(f, "{columns}")?;
        }

        if self.truncated {
            if !self.rows.is_empty() {
                writeln!(f)?;
            }

            writeln!(f, "The server truncated the result: more cards match.")?;
        }

        let mut errors = ItemErrors::default();

        for error in &self.errors {
            errors.push(error);
        }

        // NOTE: the blank line parts the errors from what printed above,
        // so none leads an output holding the errors alone.
        match self.rows.is_empty() && !self.truncated {
            true => write!(f, "{errors}"),
            false => errors.write_after(f),
        }
    }
}

/// Decodes the first `FN` of a vCard, unescaped.
fn formatted_name(data: &[u8]) -> Option<String> {
    let cst = VcardCst::parse(data).ok()?;

    cst.decode()
        .properties
        .into_iter()
        .find_map(|prop| match (prop.name, prop.value) {
            (VcardPropName::Kind(VcardPropKind::Fn), VcardValue::Text(text)) => {
                Some(text.0.into_owned())
            }
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::formatted_name;

    #[test]
    fn decodes_the_first_formatted_name() {
        let card = b"BEGIN:VCARD\r\nVERSION:4.0\r\nFN;LANGUAGE=en:Doe\\, John\r\nFN:Other\r\nEND:VCARD\r\n";
        assert_eq!(formatted_name(card).as_deref(), Some("Doe, John"));
    }

    #[test]
    fn has_no_formatted_name_without_fn() {
        let card = b"BEGIN:VCARD\r\nVERSION:4.0\r\nEND:VCARD\r\n";
        assert_eq!(formatted_name(card), None);
    }
}

#[cfg(test)]
mod report_tests {
    use anyhow::{Result, anyhow};
    use clap::Parser;
    use io_webdav::{client::WebdavClientStd, rfc4918::WebdavAuth};
    use pimalaya_cli::{error::partial_outcome_of, printer::CapturePrinter};
    use serde_json::{Value, json};
    use url::Url;

    use super::{CarddavCardEntriesOutput, EntryRow};
    use crate::{
        carddav::{
            client::CarddavClient,
            report::{multiget::CarddavReportMultigetCommand, query::CarddavReportQueryCommand},
        },
        error::ItemError,
        script::{self, Script},
    };

    /// A client over a CardDAV server answering `body` with a 207, its
    /// home set already discovered.
    fn answering(body: &str) -> CarddavClient {
        let (script, _) = Script::new([script::response(207, "application/xml", body)]);
        let base = Url::parse("https://dav.example.org/").unwrap();
        let home = base.join("/dav/").unwrap();
        let inner =
            WebdavClientStd::from_parts(script, WebdavAuth::None, base, None, None, Some(home));
        CarddavClient::new(inner)
    }

    /// The JSON a command printed and the exit status it ends with: 0, or
    /// the status of its partial outcome.
    fn run(execute: impl FnOnce(&mut CapturePrinter) -> Result<()>) -> (Value, i32) {
        let mut printer = CapturePrinter::default();
        let status = match execute(&mut printer) {
            Ok(()) => 0,
            Err(err) => partial_outcome_of(&err)
                .expect("a partial outcome")
                .status(),
        };
        (printer.into_single().unwrap(), status)
    }

    /// The `(id, code, error, sources)` of each entry of `json`'s errors.
    fn entries(json: &Value) -> Vec<Value> {
        let errors = json["errors"].as_array().unwrap();
        let fields = |entry: &Value| {
            let keys = ["id", "code", "error", "sources"];
            Value::Array(keys.iter().map(|key| entry[key].clone()).collect())
        };
        errors.iter().map(fields).collect()
    }

    const C1: &str = r#"
  <d:response>
    <d:href>/dav/book/c1.vcf</d:href>
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
  </d:response>"#;

    const C2_REFUSED: &str = r#"
  <d:response>
    <d:href>/dav/book/c2.vcf</d:href>
    <d:status>HTTP/1.1 403 Forbidden</d:status>
  </d:response>"#;

    /// A multistatus holding `responses`.
    fn multistatus(responses: &[&str]) -> String {
        let open = r#"<?xml version="1.0"?>
<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:carddav">"#;
        format!("{open}{}\n</d:multistatus>", responses.concat())
    }

    /// Each row a query answered without the card's data is an entry, as
    /// `card list` codes it; the truncation stays `truncated`, no entry,
    /// since a query `--limit` cuts on purpose.
    #[test]
    fn a_query_names_the_rows_it_could_not_read() {
        let c3 = r#"
  <d:response>
    <d:href>/dav/book/c3.vcf</d:href>
    <d:propstat>
      <d:prop><d:getetag>"e3"</d:getetag></d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
    <d:propstat>
      <d:prop><c:address-data/></d:prop>
      <d:status>HTTP/1.1 403 Forbidden</d:status>
    </d:propstat>
  </d:response>"#;
        let c4 = r#"
  <d:response>
    <d:href>/dav/book/c4.vcf</d:href>
    <d:propstat>
      <d:prop><d:getetag>"e4"</d:getetag></d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>"#;
        let truncation = r#"
  <d:response>
    <d:href>/dav/book/</d:href>
    <d:status>HTTP/1.1 507 Insufficient Storage</d:status>
  </d:response>"#;
        let body = multistatus(&[C1, C2_REFUSED, c3, c4, truncation]);

        let cmd = CarddavReportQueryCommand::try_parse_from(["query", "book"]).unwrap();
        let (json, status) = run(|printer| cmd.execute(printer, answering(&body)));

        assert_eq!(json["cards"][0]["id"], "c1.vcf");
        assert_eq!(json["cards"].as_array().unwrap().len(), 1);
        assert_eq!(json["truncated"], true);
        assert_eq!(
            entries(&json),
            [
                json!([
                    "c2.vcf",
                    "permission-denied",
                    "The server refused the card with status 403",
                    []
                ]),
                json!([
                    "c3.vcf",
                    "permission-denied",
                    "The server refused the card data",
                    ["address-data (403)"]
                ]),
                json!([
                    "c4.vcf",
                    "card-unreadable",
                    "The server sent the card without its data",
                    []
                ]),
            ]
        );
        assert_eq!(status, 3);
    }

    /// An id the server does not hold is `card-not-found`, a refused one
    /// reads as a listing row.
    #[test]
    fn a_multiget_names_the_cards_it_could_not_read() {
        let c9 = r#"
  <d:response>
    <d:href>/dav/book/c9.vcf</d:href>
    <d:status>HTTP/1.1 404 Not Found</d:status>
  </d:response>"#;
        let body = multistatus(&[C1, C2_REFUSED, c9]);

        let argv = ["multiget", "book", "c1.vcf", "c2.vcf", "c9.vcf"];
        let cmd = CarddavReportMultigetCommand::try_parse_from(argv).unwrap();
        let (json, status) = run(|printer| cmd.execute(printer, answering(&body)));

        assert_eq!(json["cards"][0]["id"], "c1.vcf");
        assert_eq!(json["cards"].as_array().unwrap().len(), 1);
        assert_eq!(
            entries(&json),
            [
                json!([
                    "c2.vcf",
                    "permission-denied",
                    "The server refused the card with status 403",
                    []
                ]),
                json!([
                    "c9.vcf",
                    "card-not-found",
                    "The server holds no such card, status 404",
                    []
                ]),
            ]
        );
        assert_eq!(status, 3);
    }

    /// Every card read: `errors` is `[]` and the command succeeds.
    #[test]
    fn a_multiget_reading_every_card_prints_no_error() {
        let body = multistatus(&[C1]);

        let cmd = CarddavReportMultigetCommand::try_parse_from(["multiget", "book", "c1.vcf"]);
        let (json, status) = run(|printer| cmd.unwrap().execute(printer, answering(&body)));

        assert_eq!(json["errors"], json!([]));
        assert_eq!(status, 0);
    }

    fn row() -> EntryRow {
        EntryRow {
            id: String::from("c1.vcf"),
            etag: Some(String::from("e1")),
            contents: String::new(),
            fn_value: Some(String::from("Jane")),
        }
    }

    /// The errors print last, after the table, the truncation and one
    /// blank line.
    #[test]
    fn the_errors_print_last_after_one_blank_line() {
        let output = CarddavCardEntriesOutput {
            rows: vec![row()],
            truncated: true,
            errors: vec![ItemError::new(Some("c2.vcf"), &anyhow!("Refused"))],
        };

        let expected = "\
ID      ETAG  FN
c1.vcf  e1    Jane

The server truncated the result: more cards match.

Error:  c2.vcf  -  Refused
";
        assert_eq!(output.to_string(), expected);
    }

    /// No blank line leads the errors when nothing prints above them.
    #[test]
    fn errors_alone_print_without_a_leading_blank_line() {
        let output = CarddavCardEntriesOutput {
            rows: Vec::new(),
            truncated: false,
            errors: vec![ItemError::new(Some("c9.vcf"), &anyhow!("Gone"))],
        };

        assert_eq!(output.to_string(), "Error:  c9.vcf  -  Gone\n");
    }
}
