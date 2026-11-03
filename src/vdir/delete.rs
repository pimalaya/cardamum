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
    /// Name of the collection (final path segment under the vdir root).
    #[arg(value_name = "NAME")]
    pub name: String,
}

impl VdirCollectionDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        let path = client.collection_path(&self.name)?;
        client.delete_collection(path)?;

        printer.out(IdReceipt {
            id: self.name,
            done: "Collection deleted",
        })
    }
}
