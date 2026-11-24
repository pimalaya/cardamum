//! # Collection rename command
//!
//! Renames a collection directory under the vdir root.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{
    shared::receipt::IdReceipt,
    vdir::client::{VdirClient, check_rename_target_absent},
};

/// Rename the given vdir collection.
///
/// A new id the root holds already is refused, coded
/// `addressbook-duplicate`.
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
        check_rename_target_absent(client.root(), &self.id, &self.new_id)?;
        client.rename_collection(path, &self.new_id)?;

        printer.out(IdReceipt {
            id: self.new_id,
            done: "Collection renamed",
        })
    }
}
