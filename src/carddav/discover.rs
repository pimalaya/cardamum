use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use serde::Serialize;

use crate::carddav::client::CarddavClient;

/// Report the resolved CardDAV endpoints for the active account.
///
/// The client resolves its server URL (RFC 6764), `current-user-principal`
/// (RFC 5397) and `addressbook-home-set` (RFC 6352) at connection time;
/// this command prints the cached results.
///
/// JSON output: `{"server", "principal", "addressbook_home_set"}`.
#[derive(Debug, Parser)]
pub struct CarddavDiscoverCommand;

impl CarddavDiscoverCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CarddavClient) -> Result<()> {
        let server = client.base_url.to_string();
        let principal = client.current_user_principal()?;
        let home = client.addressbook_home_set()?;

        printer.out(DiscoveryReport {
            server,
            principal: principal.to_string(),
            addressbook_home_set: home.to_string(),
        })
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct DiscoveryReport {
    pub server: String,
    pub principal: String,
    pub addressbook_home_set: String,
}

impl fmt::Display for DiscoveryReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "server: {}", self.server)?;
        writeln!(f, "principal: {}", self.principal)?;
        writeln!(f, "addressbook-home-set: {}", self.addressbook_home_set)
    }
}
