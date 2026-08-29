//! # Profile commands
//!
//! Dispatches the commands reading the signed-in user.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::people::{client::PeopleClient, profile::get::PeopleProfileGetCommand};

/// The signed-in user (`people/me`).
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum PeopleProfileCommand {
    Get(PeopleProfileGetCommand),
}

impl PeopleProfileCommand {
    pub fn execute(self, printer: &mut impl Printer, client: PeopleClient) -> Result<()> {
        match self {
            Self::Get(cmd) => cmd.execute(printer, client),
        }
    }
}
