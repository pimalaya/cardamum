//! # CardDAV command
//!
//! Dispatches the CardDAV-specific subcommands against the connected
//! client.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::{
    carddav::{
        client::{
            CarddavClient, check_addressbook_id, check_addressbook_name, check_card_id,
            refusal as dav_refusal,
        },
        delete::CarddavDeleteCommand,
        discover::CarddavDiscoverCommand,
        get::CarddavGetCommand,
        mkcol::CarddavMkcolCommand,
        propfind::CarddavPropfindCommand,
        proppatch::CarddavProppatchCommand,
        put::CarddavPutCommand,
        report::cli::CarddavReportCommand,
        rmcol::CarddavRmcolCommand,
    },
    shared::refusal,
};

/// CardDAV-specific API.
///
/// A flat list of WebDAV and CardDAV methods (RFC 4918, 5689, 6352,
/// 6578) run directly against the active account's endpoint, plus the
/// discovery chain. Each command is named after its protocol
/// counterpart and surfaces the native ids, ETags, CTags and sync
/// tokens the unified `addressbook` and `card` API hides.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum CarddavCommand {
    Discover(CarddavDiscoverCommand),
    Propfind(CarddavPropfindCommand),
    Proppatch(CarddavProppatchCommand),
    Mkcol(CarddavMkcolCommand),
    Rmcol(CarddavRmcolCommand),
    #[command(subcommand)]
    Report(CarddavReportCommand),
    Get(CarddavGetCommand),
    Put(CarddavPutCommand),
    Delete(CarddavDeleteCommand),
}

impl CarddavCommand {
    /// Refuses, before anything connects, an address book or card id no
    /// CardDAV resource can have, coded as the unified commands code it, and
    /// the name of a collection to create that none can have.
    pub fn check(&self) -> Result<()> {
        match self {
            Self::Discover(_) => Ok(()),
            Self::Propfind(cmd) => cmd
                .addressbook_id
                .as_deref()
                .map_or(Ok(()), check_addressbook_id),
            Self::Proppatch(cmd) => check_addressbook_id(&cmd.id),
            Self::Mkcol(cmd) => check_addressbook_name(&cmd.id),
            Self::Rmcol(cmd) => check_addressbook_id(&cmd.addressbook_id),
            Self::Report(cmd) => cmd.check(),
            Self::Get(cmd) => {
                check_addressbook_id(&cmd.addressbook_id)?;
                check_card_id(&cmd.addressbook_id, &cmd.card_id)
            }
            Self::Put(cmd) => {
                check_addressbook_id(&cmd.addressbook_id)?;
                check_card_id(&cmd.addressbook_id, &cmd.card_id)
            }
            Self::Delete(cmd) => {
                check_addressbook_id(&cmd.addressbook_id)?;
                check_card_id(&cmd.addressbook_id, &cmd.card_id)
            }
        }
    }

    pub fn execute(self, printer: &mut impl Printer, client: CarddavClient) -> Result<()> {
        let result = match self {
            Self::Discover(cmd) => cmd.execute(printer, client),
            Self::Propfind(cmd) => cmd.execute(printer, client),
            Self::Proppatch(cmd) => cmd.execute(printer, client),
            Self::Mkcol(cmd) => cmd.execute(printer, client),
            Self::Rmcol(cmd) => cmd.execute(printer, client),
            Self::Report(cmd) => cmd.execute(printer, client),
            Self::Get(cmd) => cmd.execute(printer, client),
            Self::Put(cmd) => cmd.execute(printer, client),
            Self::Delete(cmd) => cmd.execute(printer, client),
        };

        refusal::print(printer, &[dav_refusal], result)
    }
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, Parser};
    use io_webdav::{client::WebdavClientStd, rfc4918::WebdavAuth};
    use pimalaya_cli::{error::partial_outcome_of, printer::CapturePrinter};
    use serde_json::{Value, json};
    use url::Url;

    use super::CarddavCommand;
    use crate::{
        carddav::client::CarddavClient,
        cli::{Cli, Command},
        script::{self, Script},
    };

    /// Runs `cardamum carddav <args>` over a CardDAV server answering
    /// `responses`, returning its result and what it printed.
    fn run(args: &[&str], responses: Vec<Vec<u8>>) -> (anyhow::Result<()>, Option<Value>) {
        let (script, _) = Script::new(responses);
        let base = Url::parse("https://dav.example.org/").unwrap();
        let home = base.join("/dav/").unwrap();
        let inner =
            WebdavClientStd::from_parts(script, WebdavAuth::None, base, None, None, Some(home));
        let line = ["cardamum", "carddav"]
            .into_iter()
            .chain(args.iter().copied());
        let Some(Command::Carddav(cmd)) = Cli::try_parse_from(line).unwrap().cmd else {
            panic!("not a carddav command: {args:?}");
        };
        let mut printer = CapturePrinter::default();
        let result = cmd.execute(&mut printer, CarddavClient::new(inner));
        (result, printer.into_single().ok())
    }

    /// A card the server does not hold is the command's result, under
    /// `refusal`, coded `card-not-found` as the failure was, and exits 3.
    #[test]
    fn a_missing_card_is_the_result() {
        let response = script::response(404, "text/plain", "Not Found");
        let (result, printed) = run(&["get", "book", "c1.vcf"], vec![response]);

        let err = result.unwrap_err();
        assert!(partial_outcome_of(&err).is_some(), "{err:#}");
        let refusal = json!({"refusal": {
            "status": 404,
            "reason": null,
            "message": "Not Found",
            "code": "card-not-found",
            "retryAfter": null,
        }});
        assert_eq!(printed, Some(refusal));
    }

    /// A write the server forbids is the result, coded
    /// `permission-denied`.
    #[test]
    fn a_forbidden_delete_is_the_result() {
        let response = script::response(403, "text/plain", "Forbidden");
        let (result, printed) = run(&["delete", "book", "c1.vcf"], vec![response]);

        let err = result.unwrap_err();
        assert!(partial_outcome_of(&err).is_some(), "{err:#}");
        let printed = printed.unwrap();
        assert_eq!(printed["refusal"]["status"], 403, "{printed}");
        assert_eq!(printed["refusal"]["code"], "permission-denied", "{printed}");
    }

    /// A server giving no answer at all fails the command, nothing
    /// printed.
    #[test]
    fn no_answer_is_a_failure() {
        let (result, printed) = run(&["get", "book", "c1.vcf"], Vec::new());

        let err = result.unwrap_err();
        assert!(partial_outcome_of(&err).is_none(), "{err:#}");
        assert_eq!(printed, None);
    }

    /// `carddav create` runs `carddav mkcol`, flags included, hidden from
    /// the help as every former name is.
    #[test]
    fn create_is_a_hidden_alias_of_mkcol() {
        let line = ["cardamum", "carddav", "create", "book", "-n", "Book"];
        let cli = Cli::try_parse_from(line).unwrap();
        let Some(Command::Carddav(CarddavCommand::Mkcol(cmd))) = cli.cmd else {
            panic!("not a mkcol");
        };
        assert_eq!(cmd.id, "book");
        assert_eq!(cmd.name.as_deref(), Some("Book"));

        let cli = Cli::command();
        let mkcol = cli
            .find_subcommand("carddav")
            .and_then(|carddav| carddav.find_subcommand("mkcol"))
            .unwrap();
        let aliases: Vec<&str> = mkcol.get_all_aliases().collect();
        assert_eq!(aliases, ["create"]);
        assert!(mkcol.get_visible_aliases().next().is_none());
    }

    /// `carddav delete` deletes one card, its id required, so an empty
    /// shell variable never turns it into a collection delete; `rmcol`
    /// deletes the collection. `--if-none-match` takes `*` alone.
    #[test]
    fn a_card_delete_requires_its_id_and_rmcol_deletes_the_collection() {
        assert!(Cli::try_parse_from(["cardamum", "carddav", "delete", "book"]).is_err());
        assert!(Cli::try_parse_from(["cardamum", "carddav", "delete", "book", ""]).is_ok());

        let cli = Cli::try_parse_from(["cardamum", "carddav", "rmcol", "book"]).unwrap();
        let Some(Command::Carddav(CarddavCommand::Rmcol(cmd))) = cli.cmd else {
            panic!("not a rmcol");
        };
        assert_eq!(cmd.addressbook_id, "book");

        let put = [
            "cardamum",
            "carddav",
            "put",
            "book",
            "c1",
            "-",
            "--if-none-match",
        ];
        assert!(Cli::try_parse_from([&put[..], &["*"]].concat()).is_ok());
        assert!(Cli::try_parse_from([&put[..], &["\"e1\""]].concat()).is_err());
    }
}
