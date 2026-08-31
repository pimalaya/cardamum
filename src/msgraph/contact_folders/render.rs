//! # Folder rendering
//!
//! Table and JSON shapes of the contact folder commands.

use core::fmt;

use io_msgraph::v1::rest::users::contact_folders::MsgraphContactFolder;
use pimalaya_cli::table::{Cell, Color, Row, Table};
use schemars::JsonSchema;
use serde::Serialize;

use crate::shared::table::style_from_preset;

/// A page of contact folders.
///
/// The table shows ID, NAME and PARENT, while `--json` emits the raw Graph
/// folder objects plus any next-page link.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphContactFoldersOutput {
    /// The `comfy_table` preset the table is drawn with.
    #[serde(skip)]
    pub preset: String,
    /// Color of the ID column.
    #[serde(skip)]
    pub id_color: Color,
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
        let mut table = Table::new();

        table
            .load_style(style_from_preset(&self.preset))
            .set_header(Row::from([
                Cell::new("ID"),
                Cell::new("NAME"),
                Cell::new("PARENT"),
            ]))
            .add_rows(self.folders.iter().map(|folder| {
                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(Cell::new(&folder.id).fg(self.id_color))
                    .add_cell(Cell::new(&folder.display_name))
                    .add_cell(Cell::new(folder.parent_folder_id.as_deref().unwrap_or("")));
                row
            }));

        writeln!(f)?;
        write!(f, "{table}")?;
        writeln!(f)?;
        if self.next_link.is_some() {
            writeln!(f, "(more folders available: raise --top)")?;
        }
        Ok(())
    }
}

/// A single contact folder, emitted verbatim by `--json`.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct MsgraphContactFolderOutput(
    #[schemars(with = "serde_json::Value")] pub MsgraphContactFolder,
);

impl fmt::Display for MsgraphContactFolderOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let folder = &self.0;
        writeln!(f, "id: {}", folder.id)?;
        writeln!(f, "display-name: {}", folder.display_name)?;
        writeln!(
            f,
            "parent-folder-id: {}",
            folder.parent_folder_id.as_deref().unwrap_or("")
        )
    }
}
