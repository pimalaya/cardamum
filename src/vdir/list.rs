//! # Collection list command
//!
//! Lists the collections under the vdir root, as a table or as JSON.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use io_vdir::collection::VdirCollection;
use pimalaya_cli::{
    printer::Printer,
    table::{Cell, Color, Row, Table},
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{shared::table::style_from_preset, vdir::client::VdirClient};

/// List every collection under the configured vdir root.
///
/// JSON output: `{"collections": [{"id", "displayName", "description",
/// "color", "path"}]}`.
#[derive(Debug, Parser)]
pub struct VdirCollectionListCommand;

impl VdirCollectionListCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        let collections = client.list_collections()?;

        let table = VdirCollectionListOutput {
            preset: client.account.table_preset().to_string(),
            name_color: client.account.addressbooks_list_table_name_color(),
            rows: collections.into_iter().map(From::from).collect(),
        };

        printer.out(table)
    }
}

/// The collection listing, as the table and the JSON both render it.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VdirCollectionListOutput {
    /// The `comfy_table` preset the table is drawn with.
    #[serde(skip)]
    pub preset: String,
    /// Color of the NAME column.
    #[serde(skip)]
    pub name_color: Color,
    /// The collections under the vdir root.
    #[serde(rename = "collections")]
    pub rows: Vec<CollectionRow>,
}

/// One listed collection: its id, its metadata and its path on disk.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CollectionRow {
    /// Collection id, its directory name.
    pub id: String,
    /// The `displayname` file's contents, when the collection has one.
    pub display_name: Option<String>,
    /// The `description` file's contents, when it has one.
    pub description: Option<String>,
    /// The `color` file's contents, when it has one.
    pub color: Option<String>,
    /// The collection's path on disk.
    pub path: String,
}

impl From<VdirCollection> for CollectionRow {
    fn from(collection: VdirCollection) -> Self {
        Self {
            id: collection.id().to_string(),
            display_name: collection.display_name.clone(),
            description: collection.description.clone(),
            color: collection.color.clone(),
            path: collection.path.as_str().to_string(),
        }
    }
}

impl fmt::Display for VdirCollectionListOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_style(style_from_preset(&self.preset))
            .set_header(Row::from([
                Cell::new("ID"),
                Cell::new("NAME"),
                Cell::new("PATH"),
            ]))
            .add_rows(self.rows.iter().map(|c| {
                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(Cell::new(&c.id))
                    .add_cell(
                        Cell::new(c.display_name.as_deref().unwrap_or("")).fg(self.name_color),
                    )
                    .add_cell(Cell::new(&c.path));
                row
            }));

        writeln!(f)?;
        write!(f, "{table}")?;
        writeln!(f)?;
        Ok(())
    }
}
