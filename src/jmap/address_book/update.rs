//! # AddressBook update command
//!
//! Renames an AddressBook or changes its description.

use std::collections::BTreeMap;

use anyhow::Result;
use clap::Parser;
use io_jmap::rfc9610::address_book::set::{JmapAddressBookSetArgs, JmapAddressBookUpdate};
use pimalaya_cli::printer::Printer;

use crate::{
    jmap::{client::JmapClient, error::set_error},
    shared::receipt::IdReceipt,
};

/// Update a JMAP AddressBook (`AddressBook/set` update).
///
/// JSON output: `{"id"}`, the id of the AddressBook updated.
#[derive(Debug, Parser)]
pub struct JmapAddressBookUpdateCommand {
    /// AddressBook id.
    #[arg(value_name = "ID")]
    pub id: String,
    /// New name.
    #[arg(short, long, value_name = "TEXT")]
    pub name: Option<String>,
    /// New description.
    #[arg(short, long, value_name = "TEXT")]
    pub description: Option<String>,
}

impl JmapAddressBookUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: JmapClient) -> Result<()> {
        let update = BTreeMap::from([(
            self.id.clone(),
            JmapAddressBookUpdate {
                name: self.name,
                description: self.description.map(Some),
                ..Default::default()
            },
        )]);
        let args = JmapAddressBookSetArgs {
            update: Some(update),
            ..Default::default()
        };
        let out = client.address_book_set(args)?;

        if let Some(err) = out.not_updated.into_values().next() {
            return Err(set_error("AddressBook update rejected", &err));
        }

        printer.out(IdReceipt {
            id: self.id,
            done: "AddressBook updated",
        })
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use io_jmap::{client::JmapClientStd, rfc8620::session::JmapSession};
    use pimalaya_cli::printer::CapturePrinter;
    use secrecy::SecretString;
    use serde_json::{Value, json};

    use super::JmapAddressBookUpdateCommand;
    use crate::{
        jmap::client::JmapClient,
        script::{self, Script},
    };

    /// An update prints the id alone, whatever the server answers: the
    /// `/set` reply states only the properties the server changed beyond
    /// the update (RFC 8620 §5.3), `null` or an object, so the others are
    /// not known to be what a default says.
    #[test]
    fn an_update_prints_the_id_alone() {
        for updated in [Value::Null, json!({}), json!({"sortOrder": 3})] {
            let session: JmapSession = serde_json::from_value(json!({
                "username": "", "accounts": {}, "capabilities": {},
                "primaryAccounts": {"urn:ietf:params:jmap:contacts": "a1"},
                "apiUrl": "https://jmap.example.org/api/", "downloadUrl": "",
                "uploadUrl": "", "eventSourceUrl": "", "state": "",
            }))
            .unwrap();
            let body = json!({"methodResponses": [["AddressBook/set", {
                "accountId": "a1", "oldState": "s1", "newState": "s2",
                "updated": {"b7": updated},
            }, "c0"]], "sessionState": "s"});
            let response = script::response(200, "application/json", &body.to_string());
            let (script, _) = Script::new([response]);
            let auth = SecretString::from("Bearer token");
            let client = JmapClient::new(JmapClientStd::from_parts(script, auth, session));

            let args = ["update", "b7", "-n", "Work"];
            let cmd = JmapAddressBookUpdateCommand::try_parse_from(args).unwrap();
            let mut printer = CapturePrinter::default();
            cmd.execute(&mut printer, client).unwrap();

            let out = printer.into_single().unwrap();
            assert_eq!(out, json!({"id": "b7"}), "{updated}");
        }
    }
}
