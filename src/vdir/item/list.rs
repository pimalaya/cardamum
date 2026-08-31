//! # Item list command
//!
//! Lists the raw items of a collection, of any kind, as a table or JSON.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use io_vdir::item::VdirItem;
use pimalaya_cli::{
    printer::Printer,
    table::{Cell, Color, Row, Table},
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    shared::table::style_from_preset,
    vdir::{client::VdirClient, item::input::kind_str},
};

/// List every item in the given collection, of any kind.
///
/// Unlike `card list`, this includes iCalendar items and surfaces each
/// item's kind. JSON output: `{"items": [{"id", "kind", "size", "path"}]}`.
#[derive(Debug, Parser)]
pub struct VdirItemListCommand {
    /// Collection the items live in (final path segment under the root).
    #[arg(value_name = "COLLECTION")]
    pub collection: String,
}

impl VdirItemListCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        let path = client.collection_path(&self.collection)?;
        let items = client.list_items(path)?;

        let table = VdirItemListOutput {
            preset: client.account.table_preset().to_string(),
            id_color: client.account.cards_list_table_id_color(),
            rows: items.into_iter().filter_map(ItemRow::from_item).collect(),
        };

        printer.out(table)
    }
}

/// The item listing, as the table and the JSON both render it.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VdirItemListOutput {
    /// The `comfy_table` preset the table is drawn with.
    #[serde(skip)]
    pub preset: String,
    /// Color of the ID column.
    #[serde(skip)]
    pub id_color: Color,
    /// The items the collection holds.
    #[serde(rename = "items")]
    pub rows: Vec<ItemRow>,
}

/// One listed item: its id, kind, size and path on disk.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ItemRow {
    /// Item id, its file stem.
    pub id: String,
    /// The item kind its extension names.
    pub kind: &'static str,
    /// Size of its contents, in bytes.
    pub size: usize,
    /// The item's path on disk.
    pub path: String,
}

impl ItemRow {
    fn from_item(item: VdirItem) -> Option<Self> {
        let id = item.id()?.to_string();
        Some(Self {
            id,
            kind: kind_str(item.kind),
            size: item.contents.len(),
            path: item.path.as_str().to_string(),
        })
    }
}

impl fmt::Display for VdirItemListOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_style(style_from_preset(&self.preset))
            .set_header(Row::from([
                Cell::new("ID"),
                Cell::new("KIND"),
                Cell::new("SIZE"),
                Cell::new("PATH"),
            ]))
            .add_rows(self.rows.iter().map(|item| {
                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(Cell::new(&item.id).fg(self.id_color))
                    .add_cell(Cell::new(item.kind))
                    .add_cell(Cell::new(item.size))
                    .add_cell(Cell::new(&item.path));
                row
            }));

        writeln!(f)?;
        write!(f, "{table}")?;
        writeln!(f)?;
        Ok(())
    }
}
