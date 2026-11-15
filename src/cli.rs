//! # Command-line interface
//!
//! The clap parser: the global flags, the top-level command tree, and the
//! account resolution and first-run welcome every command starts from.

use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use pimalaya_cli::{
    clap::{
        args::{AccountFlag, ConfigFlags, JsonFlag, LogFlags},
        commands::{
            completion::CompletionCommand, json_schema::JsonSchemaCommand, manual::ManualCommand,
        },
    },
    error::with_code,
    footer, long_version,
    mcp::exit_on_failure,
    printer::Printer,
    welcome::{Welcome, is_wanted},
};
use pimalaya_config::toml::{ConfigMissingError, TomlConfig};

#[cfg(feature = "carddav")]
use crate::carddav::{cli::CarddavCommand, client::build_carddav_client};
#[cfg(feature = "gpeople")]
use crate::gpeople::{cli::GpeopleCommand, client::build_gpeople_client};
#[cfg(feature = "jmap")]
use crate::jmap::{cli::JmapCommand, client::build_jmap_client};
#[cfg(feature = "msgraph")]
use crate::msgraph::{cli::MsgraphCommand, client::build_msgraph_client};
#[cfg(feature = "vdir")]
use crate::vdir::{cli::VdirCommand, client::build_vdir_client};
use crate::{
    account::cli::AccountCommand,
    backend::Backend,
    config::{AccountConfig, Config},
    error::{self, ErrorCode},
    json_schema,
    mcp::McpCommand,
    repl::ReplCommand,
    session::Session,
    shared::{addressbook::cli::AddressbookCommand, card::cli::CardCommand},
};
#[cfg(feature = "pimdir")]
use crate::{
    account::context::Account,
    pimdir::{backend::PimdirBackend, cli::PimdirCommand},
};

/// The sample configuration as a raw file, for `curl` to fetch.
const SAMPLE_RAW_URL: &str =
    "https://raw.githubusercontent.com/pimalaya/cardamum/master/config.sample.toml";

/// Top-level command-line interface parser.
#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(author, version, about)]
#[command(long_version = long_version!())]
#[command(after_help = footer!())]
#[command(propagate_version = true, infer_subcommands = true)]
pub struct Cli {
    /// The subcommand to run, a bare `cardamum` printing this help.
    #[command(subcommand)]
    pub cmd: Option<Command>,
    #[command(flatten)]
    pub config: ConfigFlags,
    #[command(flatten)]
    pub account: AccountFlag,
    /// Force a specific backend for the unified commands.
    ///
    /// Only the unified commands `addressbook` and `card` consume it, the
    /// protocol-specific ones always use their own backend. The default
    /// `auto` picks the first configured backend they support, an
    /// explicit value uses that one alone and bails when the account
    /// declares no matching block or the operation has no arm for it.
    #[arg(short, long, global = true, default_value_t)]
    pub backend: Backend,
    #[command(flatten)]
    pub json: JsonFlag,
    #[command(flatten)]
    pub log: LogFlags,
}

/// The top-level commands: the unified API, one family per protocol,
/// then the meta commands.
///
/// The version it carries is what a `-V` or `--version` line of a session
/// prints, as the command line does.
#[derive(Debug, Subcommand)]
#[command(version, long_version = long_version!())]
pub enum Command {
    #[command(subcommand, alias = "addressbooks", visible_alias = "abook")]
    Addressbook(AddressbookCommand),
    #[command(subcommand, alias = "cards")]
    Card(CardCommand),
    #[cfg(feature = "carddav")]
    #[command(subcommand)]
    Carddav(CarddavCommand),
    #[cfg(feature = "jmap")]
    #[command(subcommand)]
    Jmap(JmapCommand),
    #[cfg(feature = "msgraph")]
    #[command(subcommand)]
    Msgraph(MsgraphCommand),
    #[cfg(feature = "gpeople")]
    #[command(subcommand)]
    Gpeople(GpeopleCommand),
    #[cfg(feature = "pimdir")]
    #[command(subcommand)]
    Pimdir(PimdirCommand),
    #[cfg(feature = "vdir")]
    #[command(subcommand)]
    Vdir(VdirCommand),
    Repl(ReplCommand),
    Mcp(McpCommand),
    #[command(subcommand)]
    Account(AccountCommand),
    #[command(alias = "completions")]
    Completion(CompletionCommand),
    #[command(alias = "manuals")]
    Manual(ManualCommand),
    #[command(alias = "json-schemas")]
    JsonSchema(JsonSchemaCommand),
}

/// Resolves the account a command runs against: loads the merged config,
/// then takes the account named by `-a`, or the one marked `default`.
/// Returns the leftover global config, the account name and its config.
///
/// A missing configuration fails with a [`ConfigMissingError`] naming the
/// path looked at, which is where `-c` pointed, so a mistyped path shows
/// up as itself and `main` can meet a person with the welcome.
pub fn resolve_account(
    config_paths: &[PathBuf],
    account_name: Option<&str>,
) -> Result<(Config, String, AccountConfig)> {
    let mut config = Config::load(config_paths)?;
    let (name, account_config) = config.take_account(account_name)?;
    Ok((config, name, account_config))
}

/// Meets a bare `cardamum`: the help when a configuration exists, the
/// missing configuration otherwise, which a person meets as the welcome.
pub fn meet_bare_invocation(config_paths: &[PathBuf]) -> Result<()> {
    // NOTE: a broken file counts as a configuration, its error left to
    // the first command reading it.
    if let Err(err) = Config::load(config_paths)
        && err.is::<ConfigMissingError>()
    {
        return Err(err);
    }

    Cli::command().print_help()?;
    Ok(())
}

/// The welcome a person meets when no configuration exists at `path`:
/// the commands creating it from `sample`, the shipped sample
/// configuration, or downloading it without one, then `addressbook list`.
pub fn welcome<'a>(
    path: &'a Path,
    config_paths: &'a [PathBuf],
    sample: Option<&'a Path>,
) -> Welcome<'a> {
    Welcome {
        bin: "cardamum",
        intro: "Welcome to Cardamum, the CLI to manage contacts.",
        path,
        config_paths,
        sample,
        sample_url: SAMPLE_RAW_URL,
        next: "Edit it and uncomment the options you need, then list your addressbooks:",
        command: "addressbook list",
    }
}

impl Command {
    /// Runs the command over `session`, whose account the protocol
    /// commands resolve and whose unified client the unified ones keep.
    ///
    /// A failure carries the code a typed error of its chain names
    /// ([`with_code`] over [`error::code`]), in the failure object as in a
    /// session.
    pub fn execute(self, printer: &mut impl Printer, session: &mut Session) -> Result<()> {
        self.dispatch(printer, session)
            .map_err(|err| with_code(err, error::code))
    }

    /// Runs the command, its failure as the backend returned it.
    fn dispatch(self, printer: &mut impl Printer, session: &mut Session) -> Result<()> {
        let account = session.account;
        let (config_paths, account_name) = (account.config_paths, account.name);

        match self {
            Self::Addressbook(cmd) => {
                session.run_unified(printer, |p, client| cmd.execute(p, client))
            }
            Self::Card(cmd) => cmd.execute(printer, session),

            #[cfg(feature = "carddav")]
            Self::Carddav(cmd) => {
                let (_, name, account_config) = resolve_account(config_paths, account_name)?;
                let client = build_carddav_client(name, account_config)?;
                cmd.execute(printer, client)
            }
            #[cfg(feature = "jmap")]
            Self::Jmap(cmd) => {
                let (_, name, account_config) = resolve_account(config_paths, account_name)?;
                let client = build_jmap_client(name, account_config)?;
                cmd.execute(printer, client)
            }
            #[cfg(feature = "msgraph")]
            Self::Msgraph(cmd) => {
                let (_, name, account_config) = resolve_account(config_paths, account_name)?;
                let client = build_msgraph_client(name, account_config)?;
                cmd.execute(printer, client)
            }
            #[cfg(feature = "gpeople")]
            Self::Gpeople(cmd) => {
                let (_, name, account_config) = resolve_account(config_paths, account_name)?;
                let client = build_gpeople_client(name, account_config)?;
                cmd.execute(printer, client)
            }
            #[cfg(feature = "pimdir")]
            Self::Pimdir(cmd) => {
                let (config, name, mut account_config) =
                    resolve_account(config_paths, account_name)?;
                let backend = PimdirBackend::build(name, &mut account_config)?;
                let account = Account::from(config).merge(Account::from(account_config));
                cmd.execute(printer, &account, backend)
            }
            #[cfg(feature = "vdir")]
            Self::Vdir(cmd) => {
                let (_, name, account_config) = resolve_account(config_paths, account_name)?;
                let client = build_vdir_client(name, account_config)?;
                cmd.execute(printer, client)
            }

            Self::Account(cmd) => cmd.execute(printer, config_paths, account_name, account.backend),
            Self::Repl(cmd) => cmd.execute(&*printer, session),
            Self::Mcp(cmd) => {
                // NOTE: stdout carries protocol messages only, so a failure
                // goes to stderr, a person missing a configuration aside.
                match cmd.execute(session) {
                    Err(err) if err.is::<ConfigMissingError>() && is_wanted(&*printer) => Err(err),
                    result => {
                        exit_on_failure::<ErrorCode, _>(
                            result.map_err(|err| with_code(err, error::code)),
                        );
                        Ok(())
                    }
                }
            }
            Self::Completion(cmd) => cmd.execute(printer, Cli::command()),
            Self::Manual(cmd) => cmd.execute(printer, Cli::command()),
            Self::JsonSchema(cmd) => cmd.execute(printer, json_schema::registry()),
        }
    }
}
