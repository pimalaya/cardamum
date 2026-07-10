use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::{
    addressbook::AddressbookDiff, arg::AddressbookIdArg, client::AddressbookClient,
};

/// Update an existing addressbook.
///
/// Each `--*` flag is optional and only updates the corresponding field; unset
/// fields are left untouched. To clear an optional field, pass an empty value
/// (e.g. `--description ""`).
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct AddressbookUpdateCommand {
    #[command(flatten)]
    pub addressbook: AddressbookIdArg,
    /// New human-readable name.
    #[arg(long, short, value_name = "TEXT")]
    pub name: Option<String>,
    /// New free-form description; pass `""` to clear.
    #[arg(long, short, alias = "desc", value_name = "TEXT")]
    pub description: Option<String>,
    /// New ASCII `#RRGGBB` color marker; pass `""` to clear.
    #[arg(long, short = 'C', value_name = "HEX")]
    pub color: Option<String>,
}

impl AddressbookUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: AddressbookClient) -> Result<()> {
        let patch = AddressbookDiff {
            name: self.name,
            description: self.description.map(|d| (!d.is_empty()).then_some(d)),
            color: self.color.map(|c| (!c.is_empty()).then_some(c)),
        };

        let addressbook_id = client.account.addressbook_id(self.addressbook.id)?;
        client.update_addressbook(&addressbook_id, patch)?;

        let msg = format!("Addressbook `{addressbook_id}` successfully updated");
        printer.out(Message::new(msg))
    }
}
