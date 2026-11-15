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
/// `--json` emits the raw Graph folder objects. It prints through
/// [`Paginated`], Graph's `@odata.nextLink` lifted into `nextPage`.
///
/// [`Paginated`]: pimalaya_cli::output::Paginated
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphContactFoldersOutput {
    /// The raw Graph contact folder objects.
    #[serde(rename = "folders")]
    #[schemars(with = "Vec<serde_json::Value>")]
    pub folders: Vec<MsgraphContactFolder>,
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

        write!(f, "{records}")
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
