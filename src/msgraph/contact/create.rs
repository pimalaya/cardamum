use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{
    client::MsgraphClient,
    contact::{input::ContactJsonArg, render::ContactReport},
};

/// Create a contact from a raw Graph contact JSON body.
///
/// JSON output: the raw Graph contact the server created.
#[derive(Debug, Parser)]
pub struct MsgraphContactCreateCommand {
    /// Contact folder id; omit for the default Contacts folder.
    #[arg(short = 'f', long, value_name = "FOLDER-ID")]
    pub folder: Option<String>,
    #[command(flatten)]
    pub json: ContactJsonArg,
}

impl MsgraphContactCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let contact = self.json.read()?;
        let created = client
            .contact_create(self.folder.as_deref(), &contact)?
            .response;

        printer.out(ContactReport(created))
    }
}
