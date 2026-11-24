//! # AddressBook create command
//!
//! Creates an AddressBook and prints the id the server minted.

use std::collections::BTreeMap;

use anyhow::Result;
use clap::Parser;
use io_jmap::rfc9610::address_book::set::{JmapAddressBookCreate, JmapAddressBookSetArgs};
use pimalaya_cli::printer::Printer;

use crate::{
    jmap::{client::JmapClient, error::set_error},
    shared::receipt::IdReceipt,
};

/// Create a JMAP AddressBook (`AddressBook/set` create).
///
/// JSON output: `{"id"}`, the id the server minted.
#[derive(Debug, Parser)]
pub struct JmapAddressBookCreateCommand {
    /// Name of the AddressBook.
    #[arg(value_name = "NAME")]
    pub name: String,
    /// Optional long-form description.
    #[arg(short, long, value_name = "TEXT")]
    pub description: Option<String>,
}

impl JmapAddressBookCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: JmapClient) -> Result<()> {
        let create = BTreeMap::from([(
            "c0".to_string(),
            JmapAddressBookCreate {
                name: Some(self.name),
                description: self.description,
                ..Default::default()
            },
        )]);
        let args = JmapAddressBookSetArgs {
            create: Some(create),
            ..Default::default()
        };
        let out = client.address_book_set(args)?;

        if let Some(err) = out.not_created.into_values().next() {
            return Err(set_error("AddressBook create rejected", &err));
        }
        let id = out
            .created
            .into_values()
            .next()
            .and_then(|created| created.id)
            .ok_or_else(|| anyhow::anyhow!("AddressBook create returned no id"))?;

        printer.out(IdReceipt {
            id,
            done: "AddressBook created",
        })
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use io_jmap::{client::JmapClientStd, rfc8620::session::JmapSession};
    use pimalaya_cli::printer::CapturePrinter;
    use secrecy::SecretString;
    use serde_json::json;

    use super::JmapAddressBookCreateCommand;
    use crate::{
        jmap::client::JmapClient,
        script::{self, Script},
    };

    /// A create prints the id the server minted and nothing else: the
    /// `/set` reply states only the properties the server chose (RFC 8620
    /// §5.3), so the others are not known to be what a default says.
    #[test]
    fn a_create_prints_the_id_alone() {
        let session: JmapSession = serde_json::from_value(json!({
            "username": "", "accounts": {}, "capabilities": {},
            "primaryAccounts": {"urn:ietf:params:jmap:contacts": "a1"},
            "apiUrl": "https://jmap.example.org/api/", "downloadUrl": "",
            "uploadUrl": "", "eventSourceUrl": "", "state": "",
        }))
        .unwrap();
        let body = json!({"methodResponses": [["AddressBook/set", {
            "accountId": "a1", "oldState": "s1", "newState": "s2",
            "created": {"c0": {"id": "b7"}},
        }, "c0"]], "sessionState": "s"});
        let response = script::response(200, "application/json", &body.to_string());
        let (script, _) = Script::new([response]);
        let auth = SecretString::from("Bearer token");
        let client = JmapClient::new(JmapClientStd::from_parts(script, auth, session));

        let cmd = JmapAddressBookCreateCommand::try_parse_from(["create", "Work"]).unwrap();
        let mut printer = CapturePrinter::default();
        cmd.execute(&mut printer, client).unwrap();

        assert_eq!(printer.into_single().unwrap(), json!({"id": "b7"}));
    }
}
