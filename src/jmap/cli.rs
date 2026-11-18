//! # JMAP command tree
//!
//! Nests the JMAP commands by object type and dispatches them against a
//! connected [`JmapClient`].

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::{
    jmap::{
        address_book::cli::JmapAddressBookCommand, client::JmapClient,
        contact_card::cli::JmapContactCardCommand, error::refusal as jmap_refusal,
        request::JmapRequestCommand, session::cli::JmapSessionCommand,
    },
    shared::refusal,
};

/// JMAP-specific API.
///
/// The JMAP core and contacts methods (RFC 8620, RFC 9610), nested by
/// JMAP object type, each command named after its JMAP method
/// (`/get`, `/query`, `/set` create/update/destroy, `/changes`,
/// `/copy`). Works with the raw JSContact model: `create` / `update`
/// take a JSContact JSON body and `--json` prints the raw JMAP payload.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum JmapCommand {
    #[command(subcommand, visible_aliases = ["abook", "addressbook"])]
    AddressBook(JmapAddressBookCommand),
    #[command(subcommand, visible_aliases = ["card"])]
    ContactCard(JmapContactCardCommand),
    #[command(subcommand)]
    Session(JmapSessionCommand),
    Request(JmapRequestCommand),
}

impl JmapCommand {
    pub fn execute(self, printer: &mut impl Printer, client: JmapClient) -> Result<()> {
        let result = match self {
            Self::AddressBook(cmd) => cmd.execute(printer, client),
            Self::ContactCard(cmd) => cmd.execute(printer, client),
            Self::Session(cmd) => cmd.execute(printer, client),
            Self::Request(cmd) => cmd.execute(printer, client),
        };

        refusal::print(printer, &[jmap_refusal], result)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use clap::Parser;
    use io_jmap::{client::JmapClientStd, rfc8620::session::JmapSession};
    use pimalaya_cli::{error::partial_outcome_of, printer::CapturePrinter};
    use secrecy::SecretString;
    use serde_json::{Value, json};

    use crate::{
        cli::{Cli, Command},
        jmap::client::JmapClient,
        script::{self, Script},
    };

    /// Runs `cardamum jmap <args>` over a JMAP server answering `status`
    /// with `body`, returning its result and what it printed.
    fn run(args: &[&str], status: u16, body: Value) -> (Result<()>, Option<Value>) {
        let session: JmapSession = serde_json::from_value(json!({
            "username": "", "accounts": {}, "capabilities": {},
            "primaryAccounts": {"urn:ietf:params:jmap:contacts": "a1"},
            "apiUrl": "https://jmap.example.org/api/", "downloadUrl": "",
            "uploadUrl": "", "eventSourceUrl": "", "state": "",
        }))
        .unwrap();
        let content_type = match status {
            200 => "application/json",
            _ => "application/problem+json",
        };
        let response = script::response(status, content_type, &body.to_string());
        let (script, _) = Script::new([response]);
        let auth = SecretString::from("Bearer token");
        let client = JmapClient::new(JmapClientStd::from_parts(script, auth, session));
        let line = ["cardamum", "jmap"].into_iter().chain(args.iter().copied());
        let Some(Command::Jmap(cmd)) = Cli::try_parse_from(line).unwrap().cmd else {
            panic!("not a jmap command: {args:?}");
        };
        let mut printer = CapturePrinter::default();
        let result = cmd.execute(&mut printer, client);
        (result, printer.into_single().ok())
    }

    /// The method answer `response`, as a server sends it.
    fn answer(response: Value) -> Value {
        json!({"methodResponses": [response], "sessionState": "s"})
    }

    /// A method the server answers `error` is the command's result, under
    /// `refusal`, its type the reason and its description the message, no
    /// HTTP status answering it, and exits 3.
    #[test]
    fn a_method_error_is_the_result() {
        let error = json!({"type": "invalidArguments", "description": "Bad ids."});
        let body = answer(json!(["error", error, "c0"]));
        let (result, printed) = run(&["address-book", "get", "b1"], 200, body);

        let err = result.unwrap_err();
        assert!(partial_outcome_of(&err).is_some(), "{err:#}");
        let refusal = json!({"refusal": {
            "status": null,
            "reason": "invalidArguments",
            "message": "Bad ids.",
            "code": null,
            "retryAfter": null,
        }});
        assert_eq!(printed, Some(refusal));
    }

    /// The one write of a command the server refuses is the result, its
    /// SetError type the reason, coded by it.
    #[test]
    fn a_refused_destroy_is_the_result() {
        let set = json!({
            "accountId": "a1", "oldState": "s1", "newState": "s1",
            "notDestroyed": {"c1": {"type": "forbidden"}},
        });
        let body = answer(json!(["ContactCard/set", set, "c0"]));
        let (result, printed) = run(&["contact-card", "destroy", "c1"], 200, body);

        let err = result.unwrap_err();
        assert!(partial_outcome_of(&err).is_some(), "{err:#}");
        let printed = printed.unwrap();
        assert_eq!(printed["refusal"]["status"], Value::Null, "{printed}");
        assert_eq!(printed["refusal"]["reason"], "forbidden", "{printed}");
        assert_eq!(printed["refusal"]["code"], "permission-denied", "{printed}");
    }

    /// A raw request the server refuses whole is the result, its problem
    /// type the reason, the URN namespace stripped.
    #[test]
    fn a_refused_request_is_the_result() {
        let problem = json!({
            "type": "urn:ietf:params:jmap:error:notRequest",
            "status": 400,
            "detail": "No methodCalls.",
        });
        let request = r#"{"methodCalls": []}"#;
        let (result, printed) = run(&["request", request], 400, problem);

        let err = result.unwrap_err();
        assert!(partial_outcome_of(&err).is_some(), "{err:#}");
        let refusal = json!({"refusal": {
            "status": 400,
            "reason": "notRequest",
            "message": "No methodCalls.",
            "code": null,
            "retryAfter": null,
        }});
        assert_eq!(printed, Some(refusal));
    }

    /// A copy the server applies in part names both halves: the copies
    /// made under `ids`, an item error per card refused, exit 3.
    #[test]
    fn a_copy_applied_in_part_names_both_halves() {
        let copy = json!({
            "fromAccountId": "a2", "accountId": "a1", "oldState": "s1", "newState": "s2",
            "created": {"c1": {"id": "n1"}},
            "notCreated": {"c2": {"type": "notFound"}},
        });
        let body = answer(json!(["ContactCard/copy", copy, "c0"]));
        let line = [
            "contact-card",
            "copy",
            "c1",
            "c2",
            "--from-account",
            "a2",
            "--to-address-book",
            "b1",
        ];
        let (result, printed) = run(&line, 200, body);

        let err = result.unwrap_err();
        assert!(partial_outcome_of(&err).is_some(), "{err:#}");
        let printed = printed.unwrap();
        assert_eq!(printed["ids"], json!(["n1"]), "{printed}");
        assert_eq!(printed["errors"][0]["id"], "c2", "{printed}");
        assert_eq!(printed["errors"][0]["code"], "card-not-found", "{printed}");
    }

    /// A get prints each object as the server sent it, a property
    /// io-jmap does not model kept.
    #[test]
    fn a_get_prints_the_objects_as_sent() {
        let objects = [
            (
                "address-book",
                "AddressBook",
                json!({"id": "b1", "name": "Book", "xUnmodeled": 1}),
            ),
            (
                "contact-card",
                "ContactCard",
                json!({"id": "c1", "@type": "Card", "addressBookIds": {}, "xUnmodeled": 1}),
            ),
        ];

        for (command, object, sent) in objects {
            let get = json!({"accountId": "a1", "state": "s1", "list": [sent], "notFound": []});
            let body = answer(json!([format!("{object}/get"), get, "c0"]));
            let id = sent["id"].as_str().unwrap();
            let (result, printed) = run(&[command, "get", id], 200, body);
            result.unwrap();
            let printed = printed.unwrap();
            assert_eq!(printed["list"], json!([sent]), "{command}: {printed}");
            assert_eq!(printed["state"], "s1", "{command}: {printed}");
        }
    }
}
