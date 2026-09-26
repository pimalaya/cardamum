//! # Card entries
//!
//! The card table the `query` and `multiget` REPORT commands print.

use std::fmt;

use io_webdav::rfc6352::card::CarddavCardEntry;
use pimalaya_cli::table::{Cell, Color, Row, Table};
use schemars::JsonSchema;
use serde::Serialize;
use vcard::{
    prop::{VcardPropKind, VcardPropName},
    tree::cst::VcardCst,
    value::VcardValue,
};

use crate::shared::table::style_from_preset;

/// Cards returned by a `query` or `multiget` REPORT.
///
/// The table shows ids, formatted names and ETags; the raw vCard body
/// rides in `contents` for `--json`.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CarddavCardEntriesOutput {
    /// The `comfy_table` preset the table is drawn with.
    #[serde(skip)]
    pub preset: String,
    /// Color of the ID column.
    #[serde(skip)]
    pub id_color: Color,
    /// Color of the FN column.
    #[serde(skip)]
    pub fn_color: Color,
    /// The cards the REPORT returned.
    #[serde(rename = "cards")]
    pub rows: Vec<EntryRow>,
    /// Whether the server truncated the result, more cards matching than
    /// it returned.
    pub truncated: bool,
}

/// One returned card: its id, ETag and raw vCard body.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EntryRow {
    /// Card resource id, its href last path segment.
    pub id: String,
    /// The server ETag, when it sent one.
    pub etag: Option<String>,
    /// The raw vCard, as text.
    pub contents: String,
    /// The first `FN` of the vCard, for the table only.
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
        let mut table = Table::new();

        table
            .load_style(style_from_preset(&self.preset))
            .set_header(Row::from([
                Cell::new("ID"),
                Cell::new("FN"),
                Cell::new("ETAG"),
            ]))
            .add_rows(self.rows.iter().map(|entry| {
                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(Cell::new(&entry.id).fg(self.id_color))
                    .add_cell(Cell::new(entry.fn_value.as_deref().unwrap_or("")).fg(self.fn_color))
                    .add_cell(Cell::new(entry.etag.as_deref().unwrap_or("")));
                row
            }));

        writeln!(f)?;
        write!(f, "{table}")?;
        writeln!(f)?;

        if self.truncated {
            writeln!(f, "The server truncated the result: more cards match.")?;
        }

        Ok(())
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
