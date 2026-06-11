use anyhow::Result;
use clap::Parser;
use io_addressbook::addressbook::AddressbookDiff;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::client::AddressbookClient;

/// Update an existing addressbook.
///
/// Each `--*` flag is optional and only updates the corresponding field; unset
/// fields are left untouched. To clear an optional field, pass an empty value
/// (e.g. `--description ""`).
#[derive(Debug, Parser)]
pub struct AddressbookUpdateCommand {
    /// Backend-assigned identifier of the addressbook to update.
    #[arg(value_name = "ADDRESSBOOK-ID")]
    pub id: String,
    /// New human-readable name.
    #[arg(short, long, value_name = "TEXT")]
    pub name: Option<String>,
    /// New free-form description; pass `""` to clear.
    #[arg(short, long, value_name = "TEXT")]
    pub description: Option<String>,
    /// New ASCII `#RRGGBB` color marker; pass `""` to clear.
    #[arg(short, long, value_name = "HEX")]
    pub color: Option<String>,
}

impl AddressbookUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: AddressbookClient) -> Result<()> {
        let patch = AddressbookDiff {
            name: self.name,
            description: self.description.map(|d| (!d.is_empty()).then_some(d)),
            color: self.color.map(|c| (!c.is_empty()).then_some(c)),
        };

        client.update_addressbook(&self.id, patch)?;

        let msg = format!("Addressbook `{}` successfully updated", self.id);
        printer.out(Message::new(msg))
    }
}
