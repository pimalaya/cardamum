//! # Collection rename command
//!
//! Renames a collection directory under the vdir root.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{shared::receipt::IdReceipt, vdir::client::VdirClient};

/// Rename the given vdir collection.
///
/// JSON output: `{"id"}`.
#[derive(Debug, Parser)]
pub struct VdirCollectionRenameCommand {
    /// Current collection identifier (directory name).
    #[arg(value_name = "OLD-ID")]
    pub id: String,
    /// New collection identifier (directory name).
    #[arg(value_name = "NEW-ID")]
    pub new_id: String,
}

impl VdirCollectionRenameCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        let path = client.collection_path(&self.id)?;
        client.rename_collection(path, &self.new_id)?;

        printer.out(IdReceipt {
            id: self.new_id,
            done: "Collection renamed",
        })
    }
}
