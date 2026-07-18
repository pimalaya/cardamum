use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{
    client::MsgraphClient,
    contact::{
        create::MsgraphContactCreateCommand, delete::MsgraphContactDeleteCommand,
        delta::MsgraphContactDeltaCommand, get::MsgraphContactGetCommand,
        list::MsgraphContactListCommand, update::MsgraphContactUpdateCommand,
    },
};

/// Manage Graph contacts.
///
/// `create` / `update` take a raw Graph contact JSON body (file, inline,
/// or `-` for stdin); `--json` prints the raw Graph contact.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum MsgraphContactCommand {
    List(MsgraphContactListCommand),
    Get(MsgraphContactGetCommand),
    #[command(visible_aliases = ["add", "new"])]
    Create(MsgraphContactCreateCommand),
    Update(MsgraphContactUpdateCommand),
    #[command(visible_aliases = ["del", "rm"])]
    Delete(MsgraphContactDeleteCommand),
    Delta(MsgraphContactDeltaCommand),
}

impl MsgraphContactCommand {
    pub fn execute(self, printer: &mut impl Printer, client: MsgraphClient) -> Result<()> {
        match self {
            Self::List(cmd) => cmd.execute(printer, client),
            Self::Get(cmd) => cmd.execute(printer, client),
            Self::Create(cmd) => cmd.execute(printer, client),
            Self::Update(cmd) => cmd.execute(printer, client),
            Self::Delete(cmd) => cmd.execute(printer, client),
            Self::Delta(cmd) => cmd.execute(printer, client),
        }
    }
}
