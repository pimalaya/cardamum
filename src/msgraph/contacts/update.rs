//! # Contact update command
//!
//! Patches a contact from a raw Graph contact body.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{
    client::MsgraphClient,
    contacts::{input::ContactJsonArg, render::ContactReport},
};

/// Update a contact from a raw Graph contact JSON body (PATCH: only the
/// fields present are changed).
///
/// JSON output: the raw Graph contact after the update.
#[derive(Debug, Parser)]
pub struct MsgraphContactUpdateCommand {
    /// Contact id.
    #[arg(value_name = "CONTACT-ID")]
    pub contact_id: String,
    /// The raw Graph contact JSON body.
    #[command(flatten)]
    pub json: ContactJsonArg,
}

impl MsgraphContactUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let contact = self.json.read()?;
        let updated = client.contact_update(&self.contact_id, &contact)?.response;

        printer.out(ContactReport(updated))
    }
}
