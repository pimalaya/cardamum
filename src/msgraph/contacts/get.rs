//! # Contact get command
//!
//! Reads a single contact by id.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{
    client::{MsgraphClient, contact_refusal},
    contacts::render::MsgraphContactGetOutput,
};

/// GET a contact by id.
///
/// JSON output: the contact as Graph sent it, every property kept.
#[derive(Debug, Parser)]
pub struct MsgraphContactGetCommand {
    /// Contact id.
    #[arg(value_name = "CONTACT-ID")]
    pub contact_id: String,
}

impl MsgraphContactGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let contact = client
            .raw_get(&["contacts", &self.contact_id])
            .map_err(contact_refusal)?;
        printer.out(MsgraphContactGetOutput(contact))
    }
}
