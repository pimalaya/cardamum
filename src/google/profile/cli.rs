use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::google::{client::GoogleClient, profile::get::GoogleProfileGetCommand};

/// The signed-in user (`people/me`).
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum GoogleProfileCommand {
    Get(GoogleProfileGetCommand),
}

impl GoogleProfileCommand {
    pub fn execute(self, printer: &mut impl Printer, client: GoogleClient) -> Result<()> {
        match self {
            Self::Get(cmd) => cmd.execute(printer, client),
        }
    }
}
