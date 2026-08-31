//! # Card entries
//!
//! The card table the `query` and `multiget` REPORT commands print.

use std::fmt;

use io_webdav::rfc6352::card::CarddavCardEntry;
use pimalaya_cli::table::{Cell, Color, Row, Table};
use schemars::JsonSchema;
use serde::Serialize;

use crate::shared::table::style_from_preset;

/// Cards returned by a `query` or `multiget` REPORT.
///
/// The table shows ids and ETags; the raw vCard body rides in
/// `contents` for `--json`.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CarddavCardEntriesOutput {
    /// The `comfy_table` preset the table is drawn with.
    #[serde(skip)]
    pub preset: String,
    /// Color of the ID column.
    #[serde(skip)]
    pub id_color: Color,
    /// The cards the REPORT returned.
    #[serde(rename = "cards")]
    pub rows: Vec<EntryRow>,
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
}

impl From<CarddavCardEntry> for EntryRow {
    fn from(entry: CarddavCardEntry) -> Self {
        Self {
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
            .set_header(Row::from([Cell::new("ID"), Cell::new("ETAG")]))
            .add_rows(self.rows.iter().map(|entry| {
                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(Cell::new(&entry.id).fg(self.id_color))
                    .add_cell(Cell::new(entry.etag.as_deref().unwrap_or("")));
                row
            }));

        writeln!(f)?;
        write!(f, "{table}")?;
        writeln!(f)?;
        Ok(())
    }
}
