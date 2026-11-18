//! # Microsoft Graph commands
//!
//! Dispatches the Graph-specific command tree, one subtree per Graph
//! resource.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::{
    msgraph::{
        client::{MsgraphClient, refusal as graph_refusal},
        contact_folders::cli::MsgraphContactFoldersCommand,
        contacts::cli::MsgraphContactsCommand,
        profile::cli::MsgraphProfileCommand,
        request::MsgraphRequestCommand,
    },
    shared::refusal,
};

/// Microsoft Graph-specific API.
///
/// Nested by Graph resource (contact folders, contacts, the signed-in
/// user), each command named after its Graph operation. Works with the
/// raw Graph contact model: `create` / `update` take a Graph JSON body
/// and `--json` prints the raw Graph payload.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum MsgraphCommand {
    #[command(subcommand, aliases = ["contact-folder", "folder", "folders"])]
    ContactFolders(MsgraphContactFoldersCommand),
    #[command(subcommand, alias = "contact")]
    Contacts(MsgraphContactsCommand),
    #[command(subcommand)]
    Profile(MsgraphProfileCommand),
    Request(MsgraphRequestCommand),
}

impl MsgraphCommand {
    pub fn execute(self, printer: &mut impl Printer, client: MsgraphClient) -> Result<()> {
        let result = match self {
            Self::ContactFolders(cmd) => cmd.execute(printer, client),
            Self::Contacts(cmd) => cmd.execute(printer, client),
            Self::Profile(cmd) => cmd.execute(printer, client),
            Self::Request(cmd) => cmd.execute(printer, client),
        };

        refusal::print(printer, &[graph_refusal], result)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use clap::Parser;
    use io_msgraph::v1::client::MsgraphClientStd;
    use pimalaya_cli::{error::partial_outcome_of, printer::CapturePrinter};
    use serde_json::{Value, json};

    use crate::{
        cli::{Cli, Command},
        msgraph::client::MsgraphClient,
        script::{self, Script},
    };

    /// Runs `cardamum msgraph <args>` over a Graph server answering
    /// `status` with `body`, returning its result and what it printed.
    fn run(args: &[&str], status: u16, body: Value) -> (Result<()>, Option<Value>) {
        let response = script::response(status, "application/json", &body.to_string());
        let (script, _) = Script::new([response]);
        let client =
            MsgraphClient::over(MsgraphClientStd::new(script, "token", Default::default()));
        let line = ["cardamum", "msgraph"]
            .into_iter()
            .chain(args.iter().copied());
        let Some(Command::Msgraph(cmd)) = Cli::try_parse_from(line).unwrap().cmd else {
            panic!("not a msgraph command: {args:?}");
        };
        let mut printer = CapturePrinter::default();
        let result = cmd.execute(&mut printer, client);
        (result, printer.into_single().ok())
    }

    /// A request Graph refuses is the command's result, under `refusal`,
    /// coded as a failure would be, and exits 3.
    #[test]
    fn a_refused_request_is_the_result() {
        let body = json!({"error": {"code": "ErrorAccessDenied", "message": "Access is denied."}});
        let (result, printed) = run(&["request", "get", "me/contacts/nope"], 403, body);

        let err = result.unwrap_err();
        assert!(partial_outcome_of(&err).is_some(), "{err:#}");
        let refusal = json!({"refusal": {
            "status": 403,
            "reason": "ErrorAccessDenied",
            "message": "Access is denied.",
            "code": "permission-denied",
            "retryAfter": null,
        }});
        assert_eq!(printed, Some(refusal));
    }

    /// A contact Graph does not hold is the result too, coded
    /// `card-not-found` as the failure was.
    #[test]
    fn a_missing_contact_is_the_result() {
        let body = json!({"error": {"code": "ErrorItemNotFound", "message": "Not found."}});
        let (result, printed) = run(&["contacts", "get", "AAMk"], 404, body);

        let err = result.unwrap_err();
        assert!(partial_outcome_of(&err).is_some(), "{err:#}");
        let printed = printed.unwrap();
        assert_eq!(
            printed["refusal"]["reason"], "ErrorItemNotFound",
            "{printed}"
        );
        assert_eq!(printed["refusal"]["code"], "card-not-found", "{printed}");
    }

    /// A single read prints the resource as Graph sent it, a property
    /// io-msgraph does not model and the OData annotations kept.
    #[test]
    fn a_single_read_prints_the_payload_as_sent() {
        let lines: [&[&str]; 3] = [
            &["contacts", "get", "AAMk"],
            &["contact-folders", "get", "AAMk"],
            &["profile", "get"],
        ];

        for line in lines {
            let sent = json!({
                "@odata.etag": "W/\"1\"",
                "id": "AAMk",
                "displayName": "Ada",
                "xUnmodeled": {"kept": true},
            });
            let (result, printed) = run(line, 200, sent.clone());
            result.unwrap();
            assert_eq!(printed, Some(sent), "{line:?}");
        }
    }

    /// A contact create or update sends the JSON as written, a key
    /// io-msgraph's model lacks included, and prints what Graph answered.
    #[test]
    fn a_contact_write_sends_and_prints_the_json_as_written() {
        let body = r#"{"displayName":"Ada","xUnmodeled":{"kept":true}}"#;
        let lines: [&[&str]; 2] = [
            &["contacts", "create", body],
            &["contacts", "update", "AAMk", body],
        ];

        for line in lines {
            let answer = json!({"id": "AAMk", "displayName": "Ada", "xUnmodeled": {"kept": true}});
            let response = script::response(200, "application/json", &answer.to_string());
            let (script, requests) = Script::new([response]);
            let client =
                MsgraphClient::over(MsgraphClientStd::new(script, "token", Default::default()));
            let argv = ["cardamum", "msgraph"]
                .into_iter()
                .chain(line.iter().copied());
            let Some(Command::Msgraph(cmd)) = Cli::try_parse_from(argv).unwrap().cmd else {
                panic!("not a msgraph command: {line:?}");
            };
            let mut printer = CapturePrinter::default();
            cmd.execute(&mut printer, client).unwrap();

            let request = String::from_utf8_lossy(&requests.lock().unwrap()[0]).into_owned();
            assert!(
                request.contains(r#""xUnmodeled":{"kept":true}"#),
                "{line:?}: {request}"
            );
            assert_eq!(printer.into_single().unwrap(), answer, "{line:?}");
        }
    }
}
