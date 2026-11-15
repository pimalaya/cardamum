//! # Collection delete command
//!
//! Recursively removes a collection directory under the vdir root.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{shared::receipt::IdReceipt, vdir::client::VdirClient};

/// Delete the given vdir collection.
///
/// JSON output: `{"id"}`.
#[derive(Debug, Parser)]
pub struct VdirCollectionDeleteCommand {
    /// Collection identifier (directory name).
    #[arg(value_name = "ID")]
    pub id: String,
}

impl VdirCollectionDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        let path = client.collection_path(&self.id)?;
        client.delete_collection(path)?;

        printer.out(IdReceipt {
            id: self.id,
            done: "Collection deleted",
        })
    }
}
