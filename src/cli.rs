use std::path::PathBuf;

use clap::{Parser, Subcommand};
use color_eyre::Result;
use pimalaya_tui::{
    long_version,
    terminal::{
        cli::{
            arg::path_parser,
            printer::{OutputFmt, Printer},
        },
        config::TomlConfig as _,
    },
};

use crate::{
    addressbook::command::AddressbookSubcommand, card::command::CardSubcommand,
    completion::command::GenerateCompletionsCommand, config::TomlConfig,
    manual::command::GenerateManualsCommand,
};

#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(author, version, about)]
#[command(long_version = long_version!())]
#[command(propagate_version = true, infer_subcommands = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<CardamumCommand>,

    /// Override the default configuration file path.
    ///
    /// The given paths are shell-expanded then canonicalized (if
    /// applicable). If the first path does not point to a valid file,
    /// the wizard will propose to assist you in the creation of the
    /// configuration file. Other paths are merged with the first one,
    /// which allows you to separate your public config from your
    /// private(s) one(s).
    #[arg(short, long = "config", global = true, env = "CARDAMUM_CONFIG")]
    #[arg(value_name = "PATH", value_parser = path_parser)]
    pub config_paths: Vec<PathBuf>,

    /// Customize the output format.
    ///
    /// The output format determine how to display commands output to
    /// the terminal.
    ///
    /// The possible values are:
    ///
    ///  - json: output will be in a form of a JSON-compatible object
    ///
    ///  - plain: output will be in a form of either a plain text or
    ///    table, depending on the command
    #[arg(long, short, global = true)]
    #[arg(value_name = "FORMAT", value_enum, default_value_t = Default::default())]
    pub output: OutputFmt,

    /// Disable logs.
    ///
    /// This is the same as running the command with `RUST_LOG=off`
    /// environment variable.
    #[arg(long, global = true)]
    #[arg(conflicts_with = "debug")]
    #[arg(conflicts_with = "trace")]
    pub quiet: bool,

    /// Enable debug logs with spantrace.
    ///
    /// This is the same as running the command with `RUST_LOG=debug`
    /// environment variable.
    #[arg(long, global = true)]
    #[arg(conflicts_with = "quiet")]
    #[arg(conflicts_with = "trace")]
    pub debug: bool,

    /// Enable trace logs with backtrace.
    ///
    /// This is the same as running the command with `RUST_LOG=trace`
    /// and `RUST_BACKTRACE=1` environment variables.
    #[arg(long, global = true)]
    #[arg(conflicts_with = "quiet")]
    #[arg(conflicts_with = "debug")]
    pub trace: bool,
}

#[derive(Subcommand, Debug)]
pub enum CardamumCommand {
    #[command(subcommand)]
    Addressbooks(AddressbookSubcommand),

    #[command(subcommand)]
    Cards(CardSubcommand),

    #[command(arg_required_else_help = true)]
    #[command(alias = "mans")]
    Manuals(GenerateManualsCommand),

    #[command(arg_required_else_help = true)]
    Completions(GenerateCompletionsCommand),
}

impl CardamumCommand {
    pub fn execute(self, printer: &mut impl Printer, config_paths: &[PathBuf]) -> Result<()> {
        match self {
            Self::Addressbooks(cmd) => {
                let config = TomlConfig::from_paths_or_default(config_paths)?;
                cmd.execute(printer, config)
            }
            Self::Cards(cmd) => {
                let config = TomlConfig::from_paths_or_default(config_paths)?;
                cmd.execute(printer, config)
            }
            Self::Manuals(cmd) => cmd.execute(printer),
            Self::Completions(cmd) => cmd.execute(printer),
        }
    }
}
