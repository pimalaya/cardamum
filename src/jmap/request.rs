//! # Request command
//!
//! Sends a caller-supplied JMAP request object and prints the raw
//! response.

use anyhow::{Context, Result, anyhow};
use clap::Parser;
use io_jmap::rfc8620::request::JmapRequest;
use pimalaya_cli::printer::Printer;
use serde_json::json;

use crate::{
    jmap::{client::JmapClient, input::JsonArg},
    shared::raw_json::RawJsonOutput,
};

/// Send a raw JMAP request and print the raw JMAP response.
///
/// The body is a JMAP request object (`{"using": [...], "methodCalls":
/// [...]}`), so any method can be driven with custom `using`
/// capabilities. `using` defaults to the core and contacts capabilities
/// when omitted.
///
/// JSON output: the raw JMAP response.
#[derive(Debug, Parser)]
pub struct JmapRequestCommand {
    #[command(flatten)]
    pub json: JsonArg,
}

impl JmapRequestCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: JmapClient) -> Result<()> {
        let map = self.json.read()?;

        let using = match map.get("using") {
            Some(value) => {
                serde_json::from_value(value.clone()).context("Invalid `using` array")?
            }
            None => vec![
                "urn:ietf:params:jmap:core".to_string(),
                "urn:ietf:params:jmap:contacts".to_string(),
            ],
        };
        let method_calls = map
            .get("methodCalls")
            .cloned()
            .ok_or_else(|| anyhow!("JMAP request needs a `methodCalls` array"))?;
        let method_calls =
            serde_json::from_value(method_calls).context("Invalid `methodCalls` array")?;

        let request = JmapRequest {
            using,
            method_calls,
            created_ids: None,
        };
        let response = client.send_raw(request)?;

        let value = json!({
            "methodResponses": response.method_responses,
            "createdIds": response.created_ids,
            "sessionState": response.session_state,
        });

        printer.out(RawJsonOutput(value))
    }
}
