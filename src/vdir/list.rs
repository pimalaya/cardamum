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
            // NOTE: a metadata file the collection lacks prints no line.
            let mut fields = Fields::default();
            let metadata = [
                ("Name", &collection.display_name),
                ("Description", &collection.description),
                ("Color", &collection.color),
            ];

            for (label, value) in metadata {
                if let Some(value) = value {
                    fields.push(label, sanitize::text(value));
                }
            }

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

#[cfg(test)]
mod tests {
    use super::*;

    /// A collection prints the metadata it holds, description and colour
    /// as its JSON does, and no empty line for a file it lacks.
    #[test]
    fn a_collection_prints_the_metadata_it_holds() {
        let output = VdirCollectionListOutput {
            rows: vec![
                CollectionRow {
                    id: String::from("friends"),
                    display_name: Some(String::from("Friends")),
                    description: Some(String::from("vdir made")),
                    color: Some(String::from("#ABCDEF")),
                    path: String::from("/home/vdir/friends"),
                },
                CollectionRow {
                    id: String::from("bare"),
                    display_name: None,
                    description: None,
                    color: None,
                    path: String::from("/home/vdir/bare"),
                },
            ],
        };

        let expected = "\
friends
Name:         Friends
Description:  vdir made
Color:        #ABCDEF
Path:         /home/vdir/friends

bare
Path:  /home/vdir/bare
";
        assert_eq!(output.to_string(), expected);
    }
}
