use std::{
    io::{IsTerminal, stdin},
    path::{Path, PathBuf},
};

use anyhow::{Result, bail};
use clap::{CommandFactory, Parser, Subcommand};
use pimalaya_cli::{
    clap::{
        args::{AccountFlag, JsonFlag, LogFlags},
        commands::{CompletionCommand, ManualCommand},
        parsers::path_parser,
    },
    footer, long_version,
    printer::Printer,
    prompt,
};
use pimalaya_config::toml::TomlConfig;

#[cfg(feature = "carddav")]
use crate::carddav::{cli::CarddavCommand, client::build_carddav_client};
#[cfg(feature = "jmap")]
use crate::jmap::{cli::JmapCommand, client::build_jmap_client};
#[cfg(feature = "msgraph")]
use crate::msgraph::{cli::MsgraphCommand, client::build_msgraph_client};
#[cfg(feature = "people")]
use crate::people::{cli::PeopleCommand, client::build_people_client};
#[cfg(feature = "vdir")]
use crate::vdir::{cli::VdirCommand, client::build_vdir_client};
use crate::{
    account::cli::AccountCommand,
    backend::Backend,
    config::{AccountConfig, Config},
    shared::{
        addressbook::cli::AddressbookCommand, card::cli::CardCommand, client::AddressbookClient,
    },
    wizard::{self, configure::ConfigureCommand, discover::CONFIG_SAMPLE_URL},
};

/// Top-level command-line interface parser.
#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(author, version, about)]
#[command(long_about = concat!(
    "CLI to manage contacts.\n\n",
    "First time here? Run `cardamum` with no command: it offers to generate an ",
    "account discovered from your email address, which `cardamum configure` does ",
    "again later. Everything discovery does not cover is written by hand.",
))]
#[command(long_version = long_version!())]
#[command(after_help = footer!())]
#[command(propagate_version = true, infer_subcommands = true)]
pub struct Cli {
    /// The subcommand to run.
    ///
    /// Omitted, a bare `cardamum` offers to generate a configuration when
    /// it finds none, since running the binary with no argument is what a
    /// newcomer does first, and shows this help otherwise.
    #[command(subcommand)]
    pub cmd: Option<Command>,

    /// Override the default configuration file path.
    ///
    /// The given paths are shell-expanded then canonicalized (if
    /// applicable). Other paths are merged with the first one, which
    /// allows you to separate your public config from your private
    /// one(s). Multiple paths can also be given at once, delimited by
    /// `:` like `$PATH` in a POSIX shell.
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
    /// people, vdir. With auto, the shared command picks the first
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
    #[cfg(feature = "people")]
    #[command(subcommand)]
    People(PeopleCommand),
    #[cfg(feature = "vdir")]
    #[command(subcommand)]
    Vdir(VdirCommand),

    // --- Meta
    //
    /// Configure an account interactively.
    #[command(visible_alias = "wizard")]
    Configure(ConfigureCommand),
    #[command(subcommand)]
    Account(AccountCommand),
    Completions(CompletionCommand),
    Manuals(ManualCommand),
}

/// Welcomes, then offers to generate a first configuration. Returns
/// whether the wizard ran.
///
/// Raised from the two places nothing can happen without a
/// configuration: a bare invocation, and a command that needs an
/// account. It is a hook rather than a gate, so declining it decides
/// nothing: what happens next is the caller's business, and for a
/// command that is simply carrying on.
pub fn offer_configuration(
    printer: &mut impl Printer,
    config_paths: &[PathBuf],
    path: &Path,
) -> Result<bool> {
    wizard::configure::print_welcome(path);

    if !prompt::bool("Create a configuration with a default account?", true)? {
        return Ok(false);
    }

    ConfigureCommand.execute(printer, config_paths)?;

    Ok(true)
}

/// Resolves the account a command runs against: loads the merged config
/// from `config_paths`, then takes the account named by `-a` (or the one
/// marked `default`). Returns the leftover global config, the resolved
/// account name and its config.
///
/// A missing configuration is met with the wizard rather than with an
/// error: the welcome frames what Cardamum is and offers to generate an
/// account, then the command carries on either way. Accepting is what
/// gives it a chance to work; declining leaves it to fail on the
/// configuration it still has not got. The two other failures name what
/// is missing and how to pick an account.
pub fn resolve_account(
    printer: &mut impl Printer,
    config_paths: &[PathBuf],
    account_name: Option<&str>,
) -> Result<(Config, String, AccountConfig)> {
    let mut config = match Config::from_paths_or_default(config_paths)? {
        Some(config) => config,
        None => {
            // NOTE: the target path is where `-c` pointed, or the default
            // location when it named none, so a mistyped path shows up as
            // itself rather than as a generic first run.
            let path = Config::target_path(config_paths)?;

            // NOTE: nobody is there to answer a prompt in a script or a
            // cron job, and a JSON consumer wants a failure it can read,
            // so both skip the offer and fail below.
            if !printer.is_json() && stdin().is_terminal() {
                offer_configuration(printer, config_paths, &path)?;
            }

            // NOTE: the wizard also prints the account instead of writing
            // it, so having run it proves nothing: the configuration is
            // looked up again, and the command fails the ordinary way
            // when nothing landed.
            match Config::from_paths_or_default(config_paths)? {
                Some(config) => config,
                None => bail!(
                    "No configuration found at {}, run `cardamum configure` to generate one or write it by hand: {CONFIG_SAMPLE_URL}",
                    path.display(),
                ),
            }
        }
    };

    // NOTE: an empty name and `default` both mean the default account,
    // which is the next block's business.
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
            #[cfg(feature = "people")]
            Self::People(cmd) => {
                let (config, name, account_config) =
                    resolve_account(printer, config_paths, account_name)?;
                let client = build_people_client(config, name, account_config)?;
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
            Self::Configure(cmd) => cmd.execute(printer, config_paths),
            Self::Account(cmd) => cmd.execute(printer, config_paths, account_name, backend),
            Self::Completions(cmd) => cmd.execute(printer, Cli::command()),
            Self::Manuals(cmd) => cmd.execute(printer, Cli::command()),
        }
    }
}
