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
    vdir::client::{VdirClient, is_dir_name},
};

/// Create a new vdir collection under the configured root.
///
/// JSON output: `{"id"}`.
#[derive(Debug, Parser)]
pub struct VdirCollectionCreateCommand {
    /// Name of the collection.
    ///
    /// Used both as the on-disk directory name and as the content of the
    /// `displayname` metadata file, so it is one directory name: not
    /// empty, `.` nor `..`, and holding no `/`.
    #[arg(value_name = "NAME")]
    pub name: String,
    /// Optional free-form description (`description` metadata file).
    #[arg(short, long, value_name = "TEXT")]
    pub description: Option<String>,
    /// Optional ASCII `#RRGGBB` color (`color` metadata file).
    #[arg(short = 'C', long, value_name = "HEX")]
    pub color: Option<String>,
}

impl VdirCollectionCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        if !is_dir_name(&self.name) {
            bail!("Collection name `{}` is not one directory name", self.name);
        }

        let path = client.root().join(&self.name);
        let collection = VdirCollection {
            path,
            display_name: Some(self.name.clone()),
            description: self.description,
            color: self.color,
        };

        client.create_collection(collection)?;

        printer.out(IdReceipt {
            id: self.name,
            done: "Collection created",
        })
    }
}
