//! # Contact delta command
//!
//! Runs the Graph incremental sync over contacts and renders one page of
//! changes.

use core::fmt;

use anyhow::Result;
use clap::Parser;
use io_msgraph::v1::rest::users::contacts::delta::MsgraphContactDelta;
use pimalaya_cli::{
    fields::Fields,
    printer::Printer,
    records::{Record, Records},
    sanitize,
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::msgraph::{client::MsgraphClient, contacts::render::contact_name};

/// List one page of the contacts changed or removed since the last round
/// (the Graph incremental sync).
///
/// Without `--delta-link` this opens a fresh round, enumerating everything
/// and ending with an `@odata.deltaLink`. Feed that link back through
/// `--delta-link` for the next round, which returns only what changed
/// since. A page ending with an `@odata.nextLink` was truncated, and that
/// link goes back through the same flag to drain the rest.
///
/// JSON output: `{"contacts": [<raw Graph delta>...], "@odata.nextLink",
/// "@odata.deltaLink"}`.
#[derive(Debug, Parser)]
pub struct MsgraphContactDeltaCommand {
    /// Contact folder id; omit for the default Contacts folder.
    #[arg(
        short = 'f',
        long,
        value_name = "FOLDER-ID",
        conflicts_with = "delta_link"
    )]
    pub folder: Option<String>,
    /// Comma-separated properties to return (`$select`).
    #[arg(long, value_name = "CSV", conflicts_with = "delta_link")]
    pub select: Option<String>,
    /// Continue a round from a previous page instead of opening a new one.
    ///
    /// Takes the `@odata.deltaLink` or `@odata.nextLink` that page
    /// returned. The link already carries the folder and the properties,
    /// so neither is passed again.
    #[arg(long, value_name = "URL")]
    pub delta_link: Option<String>,
}

impl MsgraphContactDeltaCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let page = match &self.delta_link {
            Some(link) => client.contacts_delta_from_link(link)?,
            None => client.contacts_delta(self.folder.as_deref(), self.select.as_deref())?,
        }
        .response;

        printer.out(MsgraphContactDeltaOutput {
            contacts: page.value,
            next_link: page.next_link,
            delta_link: page.delta_link,
        })
    }
}

/// A page of the contacts delta, with the links continuing the round.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphContactDeltaOutput {
    /// The raw Graph contact deltas of this page.
    #[serde(rename = "contacts")]
    #[schemars(with = "Vec<serde_json::Value>")]
    pub contacts: Vec<MsgraphContactDelta>,
    /// The link draining the rest of the page, when it was truncated.
    #[serde(rename = "@odata.nextLink", skip_serializing_if = "Option::is_none")]
    pub next_link: Option<String>,
    /// The link opening the next round, when the round ended here.
    #[serde(rename = "@odata.deltaLink", skip_serializing_if = "Option::is_none")]
    pub delta_link: Option<String>,
}

impl fmt::Display for MsgraphContactDeltaOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut records = Records::default();

        for delta in &self.contacts {
            let status = if delta.removed.is_some() {
                "removed"
            } else {
                "changed"
            };

            // A removed contact carries its id alone.
            let mut fields = Fields::default();
            let name = contact_name(&delta.contact);
            if !name.is_empty() {
                fields.push("Name", sanitize::text(name));
            }

            records.push(Record {
                title: sanitize::text(&delta.contact.id).into_owned(),
                state: vec![String::from(status)],
                fields,
                body: String::new(),
            });
        }

        write!(f, "{records}")?;

        if !records.items.is_empty() && (self.delta_link.is_some() || self.next_link.is_some()) {
            writeln!(f)?;
        }

        if let Some(delta_link) = &self.delta_link {
            writeln!(f, "delta-link: {}", sanitize::text(delta_link))?;
        }
        if let Some(next_link) = &self.next_link {
            writeln!(f, "next-link: {}", sanitize::text(next_link))?;
            writeln!(f, "(page truncated: pass the next-link to --delta-link)")?;
        }
        Ok(())
    }
}
