use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::vdir::{
    client::VdirClient,
    item::input::{ItemInputArg, ItemKindArg, sniff_kind},
};

/// Store a new item in the given collection.
///
/// The kind (file extension) defaults to sniffing the input
/// (`BEGIN:VCALENDAR` is iCalendar, everything else vCard); override it
/// with `--kind`. JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct VdirItemCreateCommand {
    /// Collection to store the item in.
    #[arg(value_name = "COLLECTION")]
    pub collection: String,
    /// Item kind; inferred from the input when omitted.
    #[arg(long, value_name = "KIND")]
    pub kind: Option<ItemKindArg>,
    #[command(flatten)]
    pub input: ItemInputArg,
}

impl VdirItemCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        let path = client.collection_path(&self.collection)?;
        let contents = self.input.read()?;
        let kind = match self.kind {
            Some(kind) => kind.into(),
            None => sniff_kind(&contents),
        };

        let (id, _) = client.store_item(path, None, kind, contents)?;

        printer.out(Message::new(format!("Item `{id}` successfully created")))
    }
}
