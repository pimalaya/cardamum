use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{client::MsgraphClient, contact::render::ContactReport};

/// GET a contact by id.
///
/// JSON output: the raw Graph contact object.
#[derive(Debug, Parser)]
pub struct MsgraphContactGetCommand {
    /// Contact id.
    #[arg(value_name = "CONTACT-ID")]
    pub contact_id: String,
}

impl MsgraphContactGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let contact = client.contact_get(&self.contact_id, None)?.response;
        printer.out(ContactReport(contact))
    }
}
