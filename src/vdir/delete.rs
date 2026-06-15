use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::vdir::client::VdirClient;

/// Delete the given vdir collection.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct VdirCollectionDeleteCommand {
    /// Name of the collection (final path segment under the vdir root).
    #[arg(value_name = "NAME")]
    pub name: String,
}

impl VdirCollectionDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        let path = client.root().join(&self.name);
        client.delete_collection(path)?;

        printer.out(Message::new(format!(
            "Collection `{}` successfully deleted",
            self.name
        )))
    }
}
