use std::fmt;

use comfy_table::{Cell, Color, Row, Table};
use io_webdav::rfc6352::card::CardEntry;
use serde::Serialize;

/// Cards returned by a `query` / `multiget` REPORT. The table shows id +
/// ETag; the raw vCard body rides in `contents` for `--json`.
#[derive(Clone, Debug, Serialize)]
pub struct CardEntriesReport {
    #[serde(skip)]
    pub preset: String,
    #[serde(skip)]
    pub id_color: Color,
    #[serde(rename = "cards")]
    pub rows: Vec<EntryRow>,
}

#[derive(Clone, Debug, Serialize)]
pub struct EntryRow {
    pub id: String,
    pub etag: Option<String>,
    pub contents: String,
}

impl From<CardEntry> for EntryRow {
    fn from(entry: CardEntry) -> Self {
        Self {
            id: entry.id,
            etag: entry.etag,
            contents: String::from_utf8_lossy(&entry.data).into_owned(),
        }
    }
}

impl fmt::Display for CardEntriesReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_preset(&self.preset)
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
