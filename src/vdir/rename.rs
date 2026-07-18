use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::vdir::client::VdirClient;

/// Rename the given vdir collection.
#[derive(Debug, Parser)]
pub struct VdirCollectionRenameCommand {
    /// Name of the source collection (final path segment).
    #[arg(value_name = "NAME")]
    pub source: String,
    /// New name (final path segment).
    #[arg(value_name = "TARGET")]
    pub target: String,
}

impl VdirCollectionRenameCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        let path = client.collection_path(&self.source)?;
        client.rename_collection(path, &self.target)?;

        printer.out(Message::new(format!(
            "Collection `{}` successfully renamed to `{}`",
            self.source, self.target
        )))
    }
}
