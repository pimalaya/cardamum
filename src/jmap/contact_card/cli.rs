use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::jmap::{
    client::JmapClient,
    contact_card::{
        changes::JmapContactCardChangesCommand, copy::JmapContactCardCopyCommand,
        create::JmapContactCardCreateCommand, destroy::JmapContactCardDestroyCommand,
        get::JmapContactCardGetCommand, query::JmapContactCardQueryCommand,
        update::JmapContactCardUpdateCommand,
    },
};

/// Manage JMAP ContactCards (RFC 9610 §3).
///
/// `create` / `update` take a raw JSContact JSON body (file, inline, or
/// `-` for stdin); `--json` prints the raw JMAP ContactCard.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum JmapContactCardCommand {
    Get(JmapContactCardGetCommand),
    Query(JmapContactCardQueryCommand),
    #[command(visible_aliases = ["add", "new"])]
    Create(JmapContactCardCreateCommand),
    Update(JmapContactCardUpdateCommand),
    #[command(visible_aliases = ["delete", "del", "rm", "remove"])]
    Destroy(JmapContactCardDestroyCommand),
    Changes(JmapContactCardChangesCommand),
    Copy(JmapContactCardCopyCommand),
}

impl JmapContactCardCommand {
    pub fn execute(self, printer: &mut impl Printer, client: JmapClient) -> Result<()> {
        match self {
            Self::Get(cmd) => cmd.execute(printer, client),
            Self::Query(cmd) => cmd.execute(printer, client),
            Self::Create(cmd) => cmd.execute(printer, client),
            Self::Update(cmd) => cmd.execute(printer, client),
            Self::Destroy(cmd) => cmd.execute(printer, client),
            Self::Changes(cmd) => cmd.execute(printer, client),
            Self::Copy(cmd) => cmd.execute(printer, client),
        }
    }
}
