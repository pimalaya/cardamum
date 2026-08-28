use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::msgraph::client::MsgraphClient;

/// Delete a contact by id.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct MsgraphContactDeleteCommand {
    /// Contact id.
    #[arg(value_name = "CONTACT-ID")]
    pub contact_id: String,
}

impl MsgraphContactDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        client.contact_delete(&self.contact_id)?;

        printer.out(Message::new(format!(
            "Contact `{}` successfully deleted",
            self.contact_id
        )))
    }
}
