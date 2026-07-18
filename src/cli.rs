use std::{path::PathBuf, process::exit};

use anyhow::{Result, anyhow};
use clap::{CommandFactory, Parser, Subcommand};
use pimalaya_cli::{
    clap::{
        args::{AccountFlag, JsonFlag, LogFlags},
        commands::{CompletionCommand, ManualCommand},
        parsers::path_parser,
    },
    long_version,
    printer::Printer,
    prompt,
};
use pimalaya_config::toml::TomlConfig;

#[cfg(feature = "carddav")]
use crate::carddav::{cli::CarddavCommand, client::build_carddav_client};
#[cfg(feature = "google")]
use crate::google::{cli::GoogleCommand, client::build_google_client};
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
    shared::{
        addressbook::cli::AddressbookCommand, card::cli::CardCommand, client::AddressbookClient,
    },
    wizard,
};

#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(author, version, about)]
#[command(long_version = long_version!())]
#[command(propagate_version = true, infer_subcommands = true)]
pub struct Cli {
    /// The subcommand to run. Omitted (bare `cardamum`), it runs the
    /// first-run wizard, which discovers an account and prints it as a
    /// ready-to-save config on stdout, exactly like bare `ortie`.
    #[command(subcommand)]
    pub cmd: Option<Command>,

    /// Override the default configuration file path.
    ///
    /// The given paths are shell-expanded then canonicalized (if
    /// applicable). If the first path does not point to a valid file,
    /// the wizard will propose to assist you in the creation of the
    /// configuration file. Other paths are merged with the first one,
    /// which allows you to separate your public config from your
    /// private(s) one(s). Multiple paths can also be provided by
    /// delimiting them with `:` (like `$PATH` in a POSIX shell).
    #[arg(short, long = "config", global = true, env = "CARDAMUM_CONFIG")]
    #[arg(value_name = "PATH", value_parser = path_parser, value_delimiter = ':')]
    pub config_paths: Vec<PathBuf>,
    #[command(flatten)]
    pub account: AccountFlag,
    /// Force a specific backend for cross-protocol commands.
    ///
    /// Only consumed by the shared commands (addressbook, card); the
    /// protocol-specific subcommands (vdir, carddav) ignore it and
    /// always use their own backend.
    ///
    /// Possible values: auto (default), carddav, jmap, msgraph,
    /// google, vdir. With auto, the shared command picks the first
    /// configured backend it supports; with an explicit value, it uses
    /// only that backend (and bails if the account has no matching
    /// config block, or if the operation has no implementation for
    /// it).
    #[arg(short, long, global = true, default_value_t)]
    pub backend: Backend,
    #[command(flatten)]
    pub json: JsonFlag,
    #[command(flatten)]
    pub log: LogFlags,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    // --- Shared API
    //
    #[command(subcommand, alias = "addressbooks", visible_alias = "abook")]
    Addressbook(AddressbookCommand),
    #[command(subcommand, alias = "cards")]
    Card(CardCommand),

    // --- Protocol-specific APIs
    //
    #[cfg(feature = "carddav")]
    #[command(subcommand)]
    Carddav(CarddavCommand),
    #[cfg(feature = "jmap")]
    #[command(subcommand)]
    Jmap(JmapCommand),
    #[cfg(feature = "msgraph")]
    #[command(subcommand)]
    Msgraph(MsgraphCommand),
    #[cfg(feature = "google")]
    #[command(subcommand)]
    Google(GoogleCommand),
    #[cfg(feature = "vdir")]
    #[command(subcommand)]
    Vdir(VdirCommand),

    // --- Meta
    //
    #[command(subcommand)]
    Account(AccountCommand),
    Completions(CompletionCommand),
    Manuals(ManualCommand),
}

/// Resolves the account a command runs against: loads the merged config
/// from `config_paths`, then takes the account named by `-a` (or the one
/// marked `default`). Returns the leftover global config, the resolved
/// account name and its config.
///
/// When no config file exists, proposes the first-run wizard (which
/// prints a ready-to-save config on stdout without touching disk) then
/// exits. A config that exists but lacks the requested account is a hard
/// error: `take_account` bails on a missing named account, and a missing
/// default surfaces here.
pub fn resolve_account(
    printer: &mut impl Printer,
    config_paths: &[PathBuf],
    account_name: Option<&str>,
) -> Result<(Config, String, AccountConfig)> {
    let Some(mut config) = Config::from_paths_or_default(config_paths)? else {
        if prompt::bool(
            "No configuration found. Assist you in generating one?",
            true,
        )? {
            wizard::discover::run(printer)?;
        }
        exit(0);
    };

    let (name, account_config) = config.take_account(account_name)?.ok_or_else(|| {
        anyhow!(
            "No default account found; select one with `-a <NAME>` or mark one with `default = true`"
        )
    })?;

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
            // --- Shared API
            //
            Self::Addressbook(cmd) => {
                let (config, _name, account_config) =
                    resolve_account(printer, config_paths, account_name)?;
                let client = AddressbookClient::new(config, account_config, backend)?;
                cmd.execute(printer, client)
            }
            Self::Card(cmd) => {
                let (config, _name, account_config) =
                    resolve_account(printer, config_paths, account_name)?;
                let client = AddressbookClient::new(config, account_config, backend)?;
                cmd.execute(printer, client)
            }

            // --- Protocol-specific APIs
            //
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
            #[cfg(feature = "google")]
            Self::Google(cmd) => {
                let (config, name, account_config) =
                    resolve_account(printer, config_paths, account_name)?;
                let client = build_google_client(config, name, account_config)?;
                cmd.execute(printer, client)
            }
            #[cfg(feature = "vdir")]
            Self::Vdir(cmd) => {
                let (config, name, account_config) =
                    resolve_account(printer, config_paths, account_name)?;
                let client = build_vdir_client(config, name, account_config)?;
                cmd.execute(printer, client)
            }

            // --- Meta
            //
            Self::Account(cmd) => cmd.execute(printer, config_paths, account_name, backend),
            Self::Completions(cmd) => cmd.execute(printer, Cli::command()),
            Self::Manuals(cmd) => cmd.execute(printer, Cli::command()),
        }
    }
}
