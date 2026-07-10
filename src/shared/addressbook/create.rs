use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::client::AddressbookClient;

/// Create a new addressbook.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct AddressbookCreateCommand {
    /// Human-readable name of the addressbook to create.
    #[arg(value_name = "NAME")]
    pub name: String,
    /// Optional free-form description.
    #[arg(short, long, value_name = "TEXT")]
    pub description: Option<String>,
    /// Optional ASCII `#RRGGBB` color marker.
    #[arg(short = 'C', long, value_name = "HEX")]
    pub color: Option<String>,
}

impl AddressbookCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: AddressbookClient) -> Result<()> {
        let id = client.create_addressbook(
            &self.name,
            self.description.as_deref(),
            self.color.as_deref(),
        )?;

        let msg = format!("Addressbook `{id}` successfully created");
        printer.out(Message::new(msg))
    }
}
