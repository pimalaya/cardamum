//! # Command-line interface
//!
//! The clap parser: the global flags, the top-level command tree and the
//! account resolution every command runs through.

use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::{CommandFactory, Parser, Subcommand};
use pimalaya_cli::{
    clap::{
        args::{AccountFlag, JsonFlag, LogFlags},
        commands::{CompletionCommand, JsonSchemaCommand, ManualCommand},
        parsers::path_parser,
    },
    footer, long_version,
    printer::Printer,
};
use pimalaya_config::toml::TomlConfig;

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
    config::{AccountConfig, Config, NO_CONFIG_HINT},
    json_schema,
    shared::{
        addressbook::cli::AddressbookCommand, card::cli::CardCommand, client::AddressbookClient,
    },
};
#[cfg(feature = "pimdir")]
use crate::{
    account::context::Account,
    pimdir::{backend::PimdirBackend, cli::PimdirCommand},
};

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
    /// Override the default configuration file path.
    ///
    /// Paths are shell-expanded then canonicalized. Several can be given
    /// at once, delimited by `:` like `$PATH` in a POSIX shell: the first
    /// one is the base and the rest are merged on top of it, which keeps
    /// a public configuration apart from the private ones.
    #[arg(short, long = "config", global = true, env = "CARDAMUM_CONFIG")]
    #[arg(value_name = "PATH", value_parser = path_parser, value_delimiter = ':')]
    pub config_paths: Vec<PathBuf>,
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
#[derive(Debug, Subcommand)]
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
/// Each of the four failures names what is missing and how to fix it.
pub fn resolve_account(
    _printer: &mut impl Printer,
    config_paths: &[PathBuf],
    account_name: Option<&str>,
) -> Result<(Config, String, AccountConfig)> {
    let Some(mut config) = Config::from_paths_or_default(config_paths)? else {
        bail!(
            "No configuration found at {}, {NO_CONFIG_HINT}",
            Config::target_path(config_paths)?.display(),
        );
    };

    // NOTE: an empty name and `default` both mean the default account,
    // resolved below rather than looked up by name.
    let named = account_name.filter(|name| !name.is_empty() && *name != "default");

    if let Some(name) = named.filter(|name| !config.accounts.contains_key(*name)) {
        let mut names: Vec<&str> = config.accounts.keys().map(String::as_str).collect();
        names.sort_unstable();

        bail!(
            "Account `{name}` not found, the configuration holds: {}",
            names.join(", "),
        );
    }

    let Some((name, account_config)) = config.take_account(account_name)? else {
        bail!(
            "No default account found, name one with `-a <NAME>` or mark one with `default = true`"
        );
    };

    Ok((config, name, account_config))
}

impl Command {
    pub fn execute(
        self,
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
        backend: Backend,
    ) -> Result<()> {
        match self {
            Self::Addressbook(cmd) => {
                let (config, _name, account_config) =
                    resolve_account(printer, config_paths, account_name)?;
                let client = AddressbookClient::new(config, account_config, backend)?;
                cmd.execute(printer, client)
            }
            Self::Card(cmd) => cmd.execute(printer, config_paths, account_name, backend),

            #[cfg(feature = "carddav")]
            Self::Carddav(cmd) => {
                let (config, name, account_config) =
                    resolve_account(printer, config_paths, account_name)?;
                let client = build_carddav_client(config, name, account_config)?;
                cmd.execute(printer, client)
            }
            #[cfg(feature = "jmap")]
            Self::Jmap(cmd) => {
                let (config, name, account_config) =
                    resolve_account(printer, config_paths, account_name)?;
                let client = build_jmap_client(config, name, account_config)?;
                cmd.execute(printer, client)
            }
            #[cfg(feature = "msgraph")]
            Self::Msgraph(cmd) => {
                let (config, name, account_config) =
                    resolve_account(printer, config_paths, account_name)?;
                let client = build_msgraph_client(config, name, account_config)?;
                cmd.execute(printer, client)
            }
            #[cfg(feature = "gpeople")]
            Self::Gpeople(cmd) => {
                let (config, name, account_config) =
                    resolve_account(printer, config_paths, account_name)?;
                let client = build_gpeople_client(config, name, account_config)?;
                cmd.execute(printer, client)
            }
            #[cfg(feature = "pimdir")]
            Self::Pimdir(cmd) => {
                let (config, name, mut account_config) =
                    resolve_account(printer, config_paths, account_name)?;
                let backend = PimdirBackend::build(name, &mut account_config)?;
                let account = Account::from(config).merge(Account::from(account_config));
                cmd.execute(printer, &account, backend)
            }
            #[cfg(feature = "vdir")]
            Self::Vdir(cmd) => {
                let (config, name, account_config) =
                    resolve_account(printer, config_paths, account_name)?;
                let client = build_vdir_client(config, name, account_config)?;
                cmd.execute(printer, client)
            }

            Self::Account(cmd) => cmd.execute(printer, config_paths, account_name, backend),
            Self::Completion(cmd) => cmd.execute(printer, Cli::command()),
            Self::Manual(cmd) => cmd.execute(printer, Cli::command()),
            Self::JsonSchema(cmd) => cmd.execute(printer, json_schema::registry()),
        }
    }
}
