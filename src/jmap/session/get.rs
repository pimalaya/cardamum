use core::fmt;

use anyhow::{Result, anyhow};
use clap::Parser;
use io_jmap::rfc8620::session::JmapSession;
use pimalaya_cli::printer::Printer;
use serde::Serialize;

use crate::jmap::client::JmapClient;

/// Print the JMAP session (cached at connection time).
///
/// JSON output: the raw JMAP session object.
#[derive(Debug, Parser)]
pub struct JmapSessionGetCommand;

impl JmapSessionGetCommand {
    pub fn execute(self, printer: &mut impl Printer, client: JmapClient) -> Result<()> {
        let session = client
            .session()
            .ok_or_else(|| anyhow!("JMAP session is not available"))?
            .clone();

        printer.out(SessionReport(session))
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
pub struct SessionReport(pub JmapSession);

impl fmt::Display for SessionReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let session = &self.0;
        writeln!(f, "username: {}", session.username)?;
        writeln!(f, "api-url: {}", session.api_url)?;
        writeln!(f, "state: {}", session.state)?;
        writeln!(
            f,
            "accounts: {}",
            session
                .accounts
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        writeln!(
            f,
            "capabilities: {}",
            session
                .capabilities
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
