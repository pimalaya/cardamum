//! # Discover command
//!
//! Reports the CardDAV endpoints the client resolved at connection
//! time.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{fields::Fields, printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::carddav::client::CarddavClient;

/// Report the resolved CardDAV endpoints for the active account.
///
/// The client resolves `current-user-principal` (RFC 5397) and
/// `addressbook-home-set` (RFC 6352) from its server at connection time.
/// A `home`-configured account skips the principal walk, so `principal`
/// shows as unresolved rather than failing the command.
///
/// JSON output: `{"server", "principal", "addressbookHomeSet"}`, with
/// unresolved endpoints as `null`.
#[derive(Debug, Parser)]
pub struct CarddavDiscoverCommand;

impl CarddavDiscoverCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CarddavClient) -> Result<()> {
        let server = client.base_url.to_string();
        let principal = client
            .current_user_principal()
            .ok()
            .map(|url| url.to_string());
        let addressbook_home_set = client
            .addressbook_home_set()
            .ok()
            .map(|url| url.to_string());

        printer.out(CarddavDiscoverOutput {
            server,
            principal,
            addressbook_home_set,
        })
    }
}

/// The endpoints a CardDAV session runs against, `None` when unresolved.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CarddavDiscoverOutput {
    /// The DAV context root the session runs against.
    pub server: String,
    /// The `current-user-principal` URL, `None` when unresolved.
    pub principal: Option<String>,
    /// The `addressbook-home-set` URL, `None` when unresolved.
    pub addressbook_home_set: Option<String>,
}

impl fmt::Display for CarddavDiscoverOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let unresolved = "(unresolved)";
        let principal = self.principal.as_deref().unwrap_or(unresolved);
        let home = self.addressbook_home_set.as_deref().unwrap_or(unresolved);

        let mut fields = Fields::default();
        fields.push("Server", sanitize::text(&self.server));
        fields.push("Principal", sanitize::text(principal));
        fields.push("Addressbook home-set", sanitize::text(home));
        write!(f, "{fields}")
    }
}

#[cfg(test)]
mod tests {
    use super::CarddavDiscoverOutput;

    /// The endpoints print as the family's fields, an unresolved one as
    /// such, the JSON keeping its keys.
    #[test]
    fn the_endpoints_print_as_fields() {
        let output = CarddavDiscoverOutput {
            server: String::from("https://dav.example.org/"),
            principal: None,
            addressbook_home_set: Some(String::from("https://dav.example.org/dav/")),
        };

        let text = "Server:                https://dav.example.org/\n\
                    Principal:             (unresolved)\n\
                    Addressbook home-set:  https://dav.example.org/dav/\n";
        assert_eq!(output.to_string(), text);
        assert_eq!(
            serde_json::to_string(&output).unwrap(),
            r#"{"server":"https://dav.example.org/","principal":null,"addressbookHomeSet":"https://dav.example.org/dav/"}"#
        );
    }
}
