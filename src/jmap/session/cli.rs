use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::jmap::{client::JmapClient, session::get::JmapSessionGetCommand};

/// The JMAP session object (capabilities, accounts, endpoints).
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum JmapSessionCommand {
    Get(JmapSessionGetCommand),
}

impl JmapSessionCommand {
    pub fn execute(self, printer: &mut impl Printer, client: JmapClient) -> Result<()> {
        match self {
            Self::Get(cmd) => cmd.execute(printer, client),
        }
    }
}
