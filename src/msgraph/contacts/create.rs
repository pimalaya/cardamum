//! # Contact create command
//!
//! Posts a raw Graph contact body to a contact folder.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{
    client::{MsgraphClient, folder_refusal},
    contacts::{input::ContactJsonArg, render::MsgraphContactGetOutput},
};

/// Create a contact from a raw Graph contact JSON body.
///
/// JSON output: the raw Graph contact the server created.
#[derive(Debug, Parser)]
pub struct MsgraphContactCreateCommand {
    /// Contact folder id; omit for the default Contacts folder.
    #[arg(short = 'f', long, value_name = "FOLDER-ID")]
    pub folder: Option<String>,
    /// The raw Graph contact JSON body.
    #[command(flatten)]
    pub json: ContactJsonArg,
}

impl MsgraphContactCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let contact = self.json.read()?;
        let segments = match self.folder.as_deref() {
            Some(folder) => vec!["contactFolders", folder, "contacts"],
            None => vec!["contacts"],
        };
        let created = client
            .raw_send("POST", &segments, &contact)
            .map_err(folder_refusal)?;

        printer.out(MsgraphContactGetOutput(created))
    }
}
