//! # Item move command
//!
//! Moves an item file from one collection into another.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::vdir::client::VdirClient;

/// Move an item from one collection into another.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct VdirItemMoveCommand {
    /// Source collection the item lives in.
    #[arg(value_name = "SOURCE")]
    pub source: String,
    /// Target collection to move the item into.
    #[arg(value_name = "TARGET")]
    pub target: String,
    /// Item id (file stem, without extension).
    #[arg(value_name = "ID")]
    pub id: String,
}

impl VdirItemMoveCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        let source = client.collection_path(&self.source)?;
        let target = client.collection_path(&self.target)?;
        client.move_item(source, target, &self.id)?;

        printer.out(Message::new(format!(
            "Item `{}` successfully moved from `{}` to `{}`",
            self.id, self.source, self.target
        )))
    }
}
