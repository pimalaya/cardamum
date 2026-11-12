//! # Collection list command
//!
//! Lists the collections under the vdir root, as text or as JSON.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use io_vdir::collection::VdirCollection;
use pimalaya_cli::{
    fields::Fields,
    printer::Printer,
    records::{Record, Records},
    sanitize,
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::vdir::client::VdirClient;

/// List every collection under the configured vdir root.
///
/// JSON output: `{"collections": [{"id", "displayName", "description",
/// "color", "path"}]}`.
#[derive(Debug, Parser)]
pub struct VdirCollectionListCommand;

impl VdirCollectionListCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        let collections = client.list_collections()?;

        let output = VdirCollectionListOutput {
            rows: collections.into_iter().map(From::from).collect(),
        };

        printer.out(output)
    }
}

/// The collection listing, as the text and the JSON both render it.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VdirCollectionListOutput {
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
        let mut records = Records::default();

        for collection in &self.rows {
            let name = collection.display_name.as_deref().unwrap_or_default();

            let mut fields = Fields::default();
            fields.push("Name", sanitize::text(name));
            fields.push("Path", sanitize::text(&collection.path));

            records.push(Record {
                title: sanitize::text(&collection.id).into_owned(),
                state: Vec::new(),
                fields,
                body: String::new(),
            });
        }

        write!(f, "{records}")
    }
}
