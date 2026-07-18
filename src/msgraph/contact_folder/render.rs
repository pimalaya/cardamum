use core::fmt;

use comfy_table::{Cell, Color, Row, Table};
use io_msgraph::v1::rest::users::contact_folders::MsgraphContactFolder;
use serde::Serialize;

/// A page of contact folders. The table shows ID / NAME / PARENT;
/// `--json` emits the raw Graph folder objects plus any next-page link.
#[derive(Clone, Debug, Serialize)]
pub struct FoldersReport {
    #[serde(skip)]
    pub preset: String,
    #[serde(skip)]
    pub id_color: Color,
    #[serde(rename = "folders")]
    pub folders: Vec<MsgraphContactFolder>,
    #[serde(rename = "@odata.nextLink", skip_serializing_if = "Option::is_none")]
    pub next_link: Option<String>,
}

impl fmt::Display for FoldersReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_preset(&self.preset)
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

/// A single contact folder; `--json` emits the raw Graph object.
#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
pub struct FolderReport(pub MsgraphContactFolder);

impl fmt::Display for FolderReport {
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
