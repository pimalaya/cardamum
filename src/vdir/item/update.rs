use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::vdir::{client::VdirClient, item::input::ItemInputArg};

/// Overwrite the bytes of an existing item.
///
/// The item's kind (file extension) is preserved, read from the current
/// item. JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct VdirItemUpdateCommand {
    /// Collection the item lives in.
    #[arg(value_name = "COLLECTION")]
    pub collection: String,
    /// Item id (file stem, without extension).
    #[arg(value_name = "ID")]
    pub id: String,
    #[command(flatten)]
    pub input: ItemInputArg,
}

impl VdirItemUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        let path = client.collection_path(&self.collection)?;
        let kind = client.get_item(path.clone(), &self.id)?.kind;
        let contents = self.input.read()?;

        client.store_item(path, Some(self.id.clone()), kind, contents)?;

        printer.out(Message::new(format!(
            "Item `{}` successfully updated",
            self.id
        )))
    }
}
