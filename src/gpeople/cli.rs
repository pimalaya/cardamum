//! # Google People commands
//!
//! Dispatches the People command tree to the resource module owning
//! each command.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::{
    gpeople::{
        client::{GpeopleClient, refusal as people_refusal},
        connection::cli::GpeopleConnectionCommand,
        contact_group::cli::GpeopleContactGroupCommand,
        other_contact::cli::GpeopleOtherContactCommand,
        profile::cli::GpeopleProfileCommand,
        request::GpeopleRequestCommand,
    },
    shared::refusal,
};

/// Google People-specific API.
///
/// Nested by People resource (contact groups, connections, other
/// contacts, the signed-in user), each command named after its People
/// operation. Works with the raw People model: `create` / `update` take
/// a People person JSON body and `--json` prints the raw People payload.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum GpeopleCommand {
    #[command(subcommand, alias = "contact-group", visible_aliases = ["group", "groups"])]
    ContactGroups(GpeopleContactGroupCommand),
    #[command(subcommand, alias = "connection", visible_aliases = ["people", "contacts"])]
    Connections(GpeopleConnectionCommand),
    #[command(subcommand, alias = "other-contact", visible_alias = "other")]
    OtherContacts(GpeopleOtherContactCommand),
    #[command(subcommand)]
    Profile(GpeopleProfileCommand),
    Request(GpeopleRequestCommand),
}

impl GpeopleCommand {
    pub fn execute(self, printer: &mut impl Printer, client: GpeopleClient) -> Result<()> {
        let result = match self {
            Self::ContactGroups(cmd) => cmd.execute(printer, client),
            Self::Connections(cmd) => cmd.execute(printer, client),
            Self::OtherContacts(cmd) => cmd.execute(printer, client),
            Self::Profile(cmd) => cmd.execute(printer, client),
            Self::Request(cmd) => cmd.execute(printer, client),
        };

        refusal::print(printer, &[people_refusal], result)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use clap::Parser;
    use io_gpeople::v1::client::GpeopleClientStd;
    use pimalaya_cli::{error::partial_outcome_of, printer::CapturePrinter};
    use serde_json::{Value, json};

    use crate::{
        cli::{Cli, Command},
        gpeople::client::GpeopleClient,
        script::{self, Requests, Script},
    };

    /// Runs `cardamum gpeople <args>` over a People server answering
    /// `status` with `body`, returning its result, what it printed and the
    /// requests it sent.
    fn run(args: &[&str], status: u16, body: Value) -> (Result<()>, Option<Value>, Requests) {
        let response = script::response(status, "application/json", &body.to_string());
        let (script, requests) = Script::new([response]);
        let client = GpeopleClient::over(GpeopleClientStd::new(script, "token"));
        let line = ["cardamum", "gpeople"]
            .into_iter()
            .chain(args.iter().copied());
        let Some(Command::Gpeople(cmd)) = Cli::try_parse_from(line).unwrap().cmd else {
            panic!("not a gpeople command: {args:?}");
        };
        let mut printer = CapturePrinter::default();
        let result = cmd.execute(&mut printer, client);
        (result, printer.into_single().ok(), requests)
    }

    /// A request People refuses is the command's result, under `refusal`,
    /// its first reason the provider's name for it, coded as a failure
    /// would be, and exits 3.
    #[test]
    fn a_refused_request_is_the_result() {
        let body = json!({"error": {
            "code": 403,
            "message": "The caller does not have permission",
            "status": "PERMISSION_DENIED",
            "errors": [{"reason": "forbidden", "message": "denied"}],
        }});
        let (result, printed, _) = run(&["request", "get", "people/me"], 403, body);

        let err = result.unwrap_err();
        assert!(partial_outcome_of(&err).is_some(), "{err:#}");
        let refusal = json!({"refusal": {
            "status": 403,
            "reason": "forbidden",
            "message": "The caller does not have permission",
            "code": "permission-denied",
            "retryAfter": null,
        }});
        assert_eq!(printed, Some(refusal));
    }

    /// A person People does not hold is the result too, coded
    /// `card-not-found` as the failure was, its canonical status standing
    /// for the reason it names none of.
    #[test]
    fn a_missing_person_is_the_result() {
        let body = json!({"error": {
            "code": 404,
            "message": "Requested entity was not found.",
            "status": "NOT_FOUND",
        }});
        let (result, printed, _) = run(&["connections", "get", "c9"], 404, body);

        let err = result.unwrap_err();
        assert!(partial_outcome_of(&err).is_some(), "{err:#}");
        let printed = printed.unwrap();
        assert_eq!(printed["refusal"]["reason"], "NOT_FOUND", "{printed}");
        assert_eq!(printed["refusal"]["code"], "card-not-found", "{printed}");
    }

    /// A single read prints the resource as People sent it, a property
    /// io-gpeople does not model kept, asking for the fields it asked.
    #[test]
    fn a_single_read_prints_the_payload_as_sent() {
        let lines: [(&[&str], &str); 3] = [
            (
                &["connections", "get", "c9"],
                "GET /v1/people/c9?personFields=",
            ),
            (
                &["contact-groups", "get", "g1"],
                "GET /v1/contactGroups/g1?groupFields=",
            ),
            (&["profile", "get"], "GET /v1/people/me?personFields="),
        ];

        for (line, request) in lines {
            let sent = json!({
                "resourceName": "people/c9",
                "etag": "%EgU",
                "names": [{"displayName": "Ada", "xUnmodeled": 1}],
                "xUnmodeled": {"kept": true},
            });
            let (result, printed, requests) = run(line, 200, sent.clone());
            result.unwrap();
            assert_eq!(printed, Some(sent), "{line:?}");
            let sent = String::from_utf8_lossy(&requests.lock().unwrap()[0]).into_owned();
            assert!(sent.starts_with(request), "{line:?}: {sent}");
        }
    }
}
