//! # Discover command
//!
//! Reports the CardDAV endpoints the client resolved at connection
//! time.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use serde::Serialize;

use crate::carddav::client::CarddavClient;

/// Report the resolved CardDAV endpoints for the active account.
///
/// The client resolves its server URL (RFC 6764),
/// `current-user-principal` (RFC 5397) and `addressbook-home-set` (RFC
/// 6352) at connection time. A `home`-configured account skips the
/// principal walk, so `principal` shows as unresolved rather than
/// failing the command.
///
/// JSON output: `{"server", "principal", "addressbook_home_set"}`, with
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

        printer.out(DiscoveryReport {
            server,
            principal,
            addressbook_home_set,
        })
    }
}

/// The endpoints a CardDAV session runs against, `None` when unresolved.
#[derive(Clone, Debug, Serialize)]
pub struct DiscoveryReport {
    pub server: String,
    pub principal: Option<String>,
    pub addressbook_home_set: Option<String>,
}

impl fmt::Display for DiscoveryReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let unresolved = "(unresolved)";
        writeln!(f, "server: {}", self.server)?;
        writeln!(
            f,
            "principal: {}",
            self.principal.as_deref().unwrap_or(unresolved)
        )?;
        writeln!(
            f,
            "addressbook-home-set: {}",
            self.addressbook_home_set.as_deref().unwrap_or(unresolved)
        )
    }
}
