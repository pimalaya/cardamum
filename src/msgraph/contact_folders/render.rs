//! # Folder rendering
//!
//! Text and JSON shapes of the contact folder commands.

use core::fmt;

use io_msgraph::v1::rest::users::contact_folders::MsgraphContactFolder;
use pimalaya_cli::{
    fields::Fields,
    records::{Record, Records},
    sanitize,
};
use schemars::JsonSchema;
use serde::Serialize;

/// A page of contact folders.
///
/// The text prints each folder by its id with its name and parent, while
/// `--json` emits the raw Graph folder objects plus any next-page link.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphContactFoldersOutput {
    /// The raw Graph contact folder objects.
    #[serde(rename = "folders")]
    #[schemars(with = "Vec<serde_json::Value>")]
    pub folders: Vec<MsgraphContactFolder>,
    /// The link to the next page, when the page was truncated.
    #[serde(rename = "@odata.nextLink", skip_serializing_if = "Option::is_none")]
    pub next_link: Option<String>,
}

impl fmt::Display for MsgraphContactFoldersOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut records = Records::default();

        for folder in &self.folders {
            records.push(Record {
                title: sanitize::text(&folder.id).into_owned(),
                state: Vec::new(),
                fields: folder_fields(folder),
                body: String::new(),
            });
        }

        write!(f, "{records}")?;

        if self.next_link.is_some() {
            if !records.items.is_empty() {
                writeln!(f)?;
            }

            writeln!(f, "(more folders available: raise --top)")?;
        }

        Ok(())
    }
}

/// A single contact folder, emitted verbatim by `--json`.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct MsgraphContactFolderOutput(
    #[schemars(with = "serde_json::Map<String, serde_json::Value>")] pub MsgraphContactFolder,
);

impl fmt::Display for MsgraphContactFolderOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let folder = &self.0;
        let mut fields = Fields::default();
        fields.push("Id", sanitize::text(&folder.id));
        fields.rows.extend(folder_fields(folder).rows);
        write!(f, "{fields}")
    }
}

/// The name and parent of a folder, sanitized.
fn folder_fields(folder: &MsgraphContactFolder) -> Fields<'static> {
    let parent = folder.parent_folder_id.as_deref().unwrap_or_default();

    let mut fields = Fields::default();
    fields.push("Name", sanitize::text(&folder.display_name));
    fields.push("Parent", sanitize::text(parent));
    fields
}
