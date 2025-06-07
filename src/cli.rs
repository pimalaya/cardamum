use std::path::PathBuf;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use pimalaya_toolbox::{
    config::TomlConfig,
    long_version,
    terminal::{
        clap::{
            args::{AccountArg, ConfigPathsArg, JsonFlag, LogFlags},
            commands::{CompletionCommand, ManualCommand},
        },
        printer::Printer,
    },
};

use crate::{
    // addressbook::command::AddressbookSubcommand,
    // card::command::CardSubcommand,
    addressbook::command::AddressbookSubcommand,
    config::Config,
};

#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(author, version, about)]
#[command(long_version = long_version!())]
#[command(propagate_version = true, infer_subcommands = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Cardamum,
    #[command(flatten)]
    pub config: ConfigPathsArg,
    #[command(flatten)]
    pub account: AccountArg,
    #[command(flatten)]
    pub json: JsonFlag,
    #[command(flatten)]
    pub log: LogFlags,
}

#[derive(Subcommand, Debug)]
pub enum Cardamum {
    #[command(arg_required_else_help = true, subcommand)]
    Addressbooks(AddressbookSubcommand),
    // #[command(arg_required_else_help = true, subcommand)]
    // Cards(CardSubcommand),
    #[command(arg_required_else_help = true, alias = "mans")]
    Manuals(ManualCommand),
    #[command(arg_required_else_help = true)]
    Completions(CompletionCommand),
}

impl Cardamum {
    pub fn execute(
        self,
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
    ) -> Result<()> {
        match self {
            Self::Addressbooks(cmd) => {
                let config = Config::from_paths_or_default(config_paths)?;
                let (_, account) = config.get_account(account_name)?;
                cmd.execute(printer, account)
            }
            // Self::Cards(cmd) => {
            //     let config = TomlConfig::from_paths_or_default(config_paths)?;
            //     cmd.execute(printer, config)
            // }
            Self::Manuals(cmd) => cmd.execute(printer, Cli::command()),
            Self::Completions(cmd) => cmd.execute(printer, Cli::command()),
        }
    }
}
