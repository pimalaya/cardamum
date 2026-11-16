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
    backend::{self, Backend},
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
    /// Force a specific backend for cross-protocol commands.
    ///
    /// Only the unified commands read it, the protocol-specific ones always
    /// using their own backend. `auto`, the default, picks the first
    /// configured backend the command supports; an explicit value bails when
    /// the account has no such block, or the command no such implementation.
    #[arg(short, long, global = true, default_value_t, value_parser = backend::parser())]
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
                cmd.check()?;
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
                let backend = PimdirBackend::build(name.clone(), &mut account_config)?;
                let account = Account::from(config).merge(Account::from(account_config));
                cmd.execute(printer, &name, &account, backend)
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

#[cfg(all(test, feature = "pimdir"))]
mod tests {
    use clap::Parser;

    use crate::{
        cli::{Cli, Command},
        pimdir::cli::PimdirCommand::{Count, Search},
    };

    fn parse(args: &[&str]) -> clap::error::Result<Cli> {
        Cli::try_parse_from([&["cardamum"][..], args].concat())
    }

    /// An option after a `pimdir search` or `pimdir count` query is read
    /// as the option, never as a query word, `--json` included; a word
    /// starting with `-` passes after `--`, and is refused before it.
    #[test]
    fn an_option_after_a_query_is_an_option() {
        let cli = parse(&["pimdir", "search", "alice", "--page-size", "3"]).unwrap();
        let cli = [
            cli,
            parse(&["pimdir", "search", "alice", "-s", "3", "--json"]).unwrap(),
        ];
        for (i, cli) in cli.into_iter().enumerate() {
            assert_eq!(cli.json.enabled, i == 1);
            let Some(Command::Pimdir(Search(search))) = cli.cmd else {
                panic!("not a search");
            };
            assert_eq!(search.query, ["alice"]);
            assert_eq!(search.page_size, Some(3));
        }

        let cli = parse(&["pimdir", "count", "alice", "-k", "book"]).unwrap();
        let Some(Command::Pimdir(Count(count))) = cli.cmd else {
            panic!("not a count");
        };
        assert_eq!(count.query, ["alice"]);
        assert_eq!(count.addressbook.as_deref(), Some("book"));

        let cli = parse(&["pimdir", "search", "-s", "5", "--", "alice", "-zoe"]).unwrap();
        let Some(Command::Pimdir(Search(search))) = cli.cmd else {
            panic!("not a search");
        };
        assert_eq!(search.query, ["alice", "-zoe"]);
        assert_eq!(search.page_size, Some(5));

        assert!(parse(&["pimdir", "search", "alice", "-zoe"]).is_err());
        assert!(parse(&["pimdir", "count", "alice", "-zoe"]).is_err());
    }
}

#[cfg(test)]
mod usage_tests {
    use clap::{CommandFactory, Parser, error::ErrorKind};

    use crate::{
        backend::{Backend, NAMES},
        cli::Cli,
    };

    /// The short names of `addressbook delete`, `card add`, `card delete`
    /// and `msgraph contact-folders` still run, hidden from the help as
    /// calendula shows none for their counterparts.
    #[test]
    fn the_short_names_are_hidden_aliases() {
        let mut aliases = vec![
            (["addressbook", "delete"], "rm"),
            (["card", "add"], "new"),
            (["card", "delete"], "rm"),
        ];

        if cfg!(feature = "msgraph") {
            aliases.push((["msgraph", "contact-folders"], "folders"));
        }

        let mut cli = Cli::command();
        cli.build();

        for ([group, leaf], alias) in aliases {
            let cmd = cli
                .find_subcommand(group)
                .and_then(|group| group.find_subcommand(leaf))
                .unwrap();
            assert!(cmd.get_all_aliases().any(|name| name == alias), "{leaf}");
            assert!(cmd.get_visible_aliases().next().is_none(), "{leaf}");
        }
    }

    /// The Graph listings show `--top` long-only, as Himalaya's do, and
    /// still read the former `-s` as `--top`.
    #[cfg(feature = "msgraph")]
    #[test]
    fn the_graph_top_is_long_only() {
        let listings = [
            ["msgraph", "contacts", "list"],
            ["msgraph", "contact-folders", "list"],
            ["msgraph", "contact-folders", "child-folders"],
        ];

        let mut cli = Cli::command();
        cli.build();

        for [group, resource, leaf] in listings {
            let mut cmd = cli
                .find_subcommand(group)
                .and_then(|group| group.find_subcommand(resource))
                .and_then(|resource| resource.find_subcommand(leaf))
                .unwrap()
                .clone();
            let help = cmd.render_long_help().to_string();
            assert!(help.contains("--top <N>"), "{leaf}: {help}");
            assert!(!help.contains("-s,"), "{leaf}: {help}");

            let mut args = vec!["cardamum", group, resource, leaf, "-s", "5"];
            if leaf == "child-folders" {
                args.push("folder");
            }

            let matches = cli.clone().try_get_matches_from(args).unwrap();
            let top = matches
                .subcommand_matches(group)
                .and_then(|matches| matches.subcommand_matches(resource))
                .and_then(|matches| matches.subcommand_matches(leaf))
                .and_then(|matches| matches.get_one::<u32>("top"))
                .copied();
            assert_eq!(top, Some(5), "{leaf}");
        }
    }

    /// `-b` reads as Himalaya's, and a value it does not know is refused
    /// with the possible ones, every backend an account may name.
    #[test]
    fn the_backend_flag_lists_its_values() {
        let mut cli = Cli::command();
        cli.build();
        let flag = cli
            .get_arguments()
            .find(|arg| arg.get_id() == "backend")
            .unwrap();

        let help = flag.get_help().unwrap().to_string();
        assert_eq!(help, "Force a specific backend for cross-protocol commands");

        let values: Vec<String> = flag
            .get_possible_values()
            .iter()
            .map(|value| value.get_name().to_string())
            .collect();
        assert_eq!(values[0], "auto");
        assert_eq!(values[1..], NAMES);

        let err = Cli::try_parse_from(["cardamum", "-b", "nope", "card", "list"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
        let message = err.to_string();
        assert!(
            message
                .contains("[possible values: auto, vdir, pimdir, carddav, jmap, msgraph, gpeople]"),
            "{message}"
        );

        let cli = Cli::try_parse_from(["cardamum", "-b", "auto", "card", "list"]).unwrap();
        assert_eq!(cli.backend, Backend::Auto);
    }

    /// A backend this build leaves out is a possible value still, refused
    /// naming the feature it needs.
    #[cfg(not(feature = "jmap"))]
    #[test]
    fn a_backend_left_out_names_its_feature() {
        let err = Cli::try_parse_from(["cardamum", "-b", "jmap", "card", "list"]).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("--features jmap"), "{message}");
    }

    /// Each protocol namespace this build carries reads as Himalaya's,
    /// `<API>-specific API`.
    #[test]
    fn the_namespaces_read_as_himalaya() {
        let abouts = [
            ("carddav", "CardDAV-specific API"),
            ("jmap", "JMAP-specific API"),
            ("msgraph", "Microsoft Graph-specific API"),
            ("gpeople", "Google People-specific API"),
            ("pimdir", "pimdir-specific API"),
            ("vdir", "vdir-specific API"),
        ];

        let mut cli = Cli::command();
        cli.build();

        for (name, about) in abouts {
            if let Some(cmd) = cli.find_subcommand(name) {
                let found = cmd.get_about().map(ToString::to_string);
                assert_eq!(found.as_deref(), Some(about), "{name}");
            }
        }
    }

    /// A backend this build carries is taken by its name.
    #[cfg(feature = "vdir")]
    #[test]
    fn a_built_backend_is_taken_by_its_name() {
        let cli = Cli::try_parse_from(["cardamum", "-b", "vdir", "card", "list"]).unwrap();
        assert_eq!(cli.backend, Backend::Vdir);
    }
}
