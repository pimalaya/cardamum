mod account;
mod backend;
#[cfg(feature = "carddav")]
mod carddav;
mod cli;
mod config;
#[cfg(feature = "google")]
mod google;
#[cfg(feature = "jmap")]
mod jmap;
#[cfg(feature = "msgraph")]
mod msgraph;
#[cfg(any(feature = "msgraph", feature = "google"))]
mod project;
mod shared;
#[cfg(feature = "vdir")]
mod vdir;
mod wizard;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{error::ErrorReport, log::Logger, printer::StdoutPrinter};

use crate::cli::Cli;

fn main() {
    let cli = Cli::parse();
    let mut printer = StdoutPrinter::new(&cli.json);
    let result = execute(cli, &mut printer);
    ErrorReport::eval(&mut printer, result);
}

fn execute(cli: Cli, printer: &mut StdoutPrinter) -> Result<()> {
    Logger::try_init(&cli.log)?;
    let config = cli.config_paths.as_ref();
    let account = cli.account.name.as_deref();
    let backend = cli.backend;
    cli.cmd.execute(printer, config, account, backend)
}
