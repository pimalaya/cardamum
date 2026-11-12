//! # Session get command
//!
//! Prints the JMAP session object cached when the client connected.

use core::fmt;

use anyhow::{Result, anyhow};
use clap::Parser;
use io_jmap::rfc8620::session::JmapSession;
use pimalaya_cli::{fields::Fields, printer::Printer, sanitize};
use schemars::JsonSchema;
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

        printer.out(JmapSessionGetOutput(session))
    }
}

/// The JMAP session, as a summary block or as the raw object.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct JmapSessionGetOutput(
    #[schemars(with = "serde_json::Map<String, serde_json::Value>")] pub JmapSession,
);

impl fmt::Display for JmapSessionGetOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let session = &self.0;
        let accounts = session.accounts.keys().cloned().collect::<Vec<_>>();
        let capabilities = session.capabilities.keys().cloned().collect::<Vec<_>>();

        let mut fields = Fields::default();
        fields.push("Username", sanitize::text(&session.username));
        fields.push("API URL", sanitize::text(session.api_url.as_ref()));
        fields.push("State", sanitize::text(&session.state));
        fields.push("Accounts", sanitize::text(&accounts.join(", ")));
        fields.push("Capabilities", sanitize::text(&capabilities.join(", ")));
        write!(f, "{fields}")
    }
}
