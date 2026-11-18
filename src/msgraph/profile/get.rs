//! # Profile get command
//!
//! Reads the signed-in user and renders it.

use core::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{fields::Fields, printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::msgraph::client::MsgraphClient;

/// GET the signed-in user (Graph `/me`).
///
/// JSON output: the Graph user object as Graph sent it, every property
/// kept.
#[derive(Debug, Parser)]
pub struct MsgraphProfileGetCommand;

impl MsgraphProfileGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let user = client.raw_get(&[])?;
        printer.out(MsgraphProfileGetOutput(user))
    }
}

/// The signed-in user as Graph sent it, every property kept, rendered as
/// aligned text or, under `--json`, verbatim.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct MsgraphProfileGetOutput(#[schemars(with = "Map<String, Value>")] pub Value);

impl fmt::Display for MsgraphProfileGetOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fields = Fields::default();

        for (label, key) in [
            ("Id", "id"),
            ("Display name", "displayName"),
            ("Mail", "mail"),
            ("User principal name", "userPrincipalName"),
        ] {
            let value = self.0.get(key).and_then(Value::as_str).unwrap_or_default();
            fields.push(label, sanitize::text(value));
        }

        write!(f, "{fields}")
    }
}
