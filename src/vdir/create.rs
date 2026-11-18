//! # Collection create command
//!
//! Creates a collection directory under the vdir root, with the metadata
//! files describing it.

use anyhow::{Result, bail};
use clap::Parser;
use io_vdir::collection::VdirCollection;
use pimalaya_cli::printer::Printer;

use crate::{
    shared::receipt::IdReceipt,
    vdir::client::{VdirClient, check_collection_absent, is_dir_name},
};

/// Create a new vdir collection under the configured root.
///
/// The directory is made under the configured root, and each metadata
/// flag below writes the file vdir reads that value from. An id the root
/// holds already is refused, coded `addressbook-duplicate`.
///
/// JSON output: `{"id"}`.
#[derive(Debug, Parser)]
pub struct VdirCollectionCreateCommand {
    /// Collection identifier, its directory name: not empty, `.` nor `..`,
    /// and holding no `/`.
    #[arg(value_name = "ID")]
    pub id: String,
    /// Display name, written to the `displayname` metadata file; the id
    /// when omitted.
    #[arg(short = 'n', long, value_name = "NAME")]
    pub display_name: Option<String>,
    /// Optional free-form description (`description` metadata file).
    #[arg(short, long, value_name = "TEXT")]
    pub description: Option<String>,
    /// Optional ASCII `#RRGGBB` color (`color` metadata file).
    #[arg(short = 'C', long, value_name = "HEX")]
    pub color: Option<String>,
}

impl VdirCollectionCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        if !is_dir_name(&self.id) {
            bail!("Collection id `{}` is not one directory name", self.id);
        }

        check_collection_absent(client.root(), &self.id)?;

        let path = client.root().join(&self.id);
        let collection = VdirCollection {
            path,
            display_name: Some(self.display_name.unwrap_or_else(|| self.id.clone())),
            description: self.description,
            color: self.color,
        };

        client.create_collection(collection)?;

        printer.out(IdReceipt {
            id: self.id,
            done: "Collection created",
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use clap::Parser;
    use pimalaya_cli::printer::CapturePrinter;
    use serde_json::json;

    use super::VdirCollectionCreateCommand;
    use crate::{config::VdirConfig, vdir::client::VdirClient};

    /// `-n` writes the display name apart from the id, as calendula's
    /// `vdir create` does; without it the id stays the display name.
    #[test]
    fn the_display_name_is_its_own_flag() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().display().to_string();
        let displayname = |id: &str| fs::read_to_string(dir.path().join(id).join("displayname"));

        for (argv, id, name) in [
            (
                &["create", "friends", "-n", "Friends"][..],
                "friends",
                "Friends",
            ),
            (
                &["create", "work", "--display-name", "Work"][..],
                "work",
                "Work",
            ),
            (&["create", "bare"][..], "bare", "bare"),
        ] {
            let cmd = VdirCollectionCreateCommand::try_parse_from(argv).unwrap();
            let client = VdirClient::new(VdirConfig { root: root.clone() });
            let mut printer = CapturePrinter::default();
            cmd.execute(&mut printer, client).unwrap();

            assert_eq!(printer.into_single().unwrap(), json!({ "id": id }));
            assert_eq!(displayname(id).unwrap().trim_end(), name, "{argv:?}");
        }
    }
}
