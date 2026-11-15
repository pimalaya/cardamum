//! # Contact delta command
//!
//! Runs the Graph incremental sync over contacts and renders one page of
//! changes.

use core::fmt;

use anyhow::Result;
use clap::Parser;
use io_msgraph::v1::rest::users::contacts::delta::MsgraphContactDelta;
use pimalaya_cli::{
    clap::args::PageTokenArg,
    fields::Fields,
    output::Paginated,
    printer::Printer,
    records::{Record, Records},
    sanitize,
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::msgraph::{
    client::{MsgraphClient, folder_refusal},
    contacts::render::contact_name,
};

/// List one page of the contacts changed or removed since the last round
/// (the Graph incremental sync).
///
/// Without `--delta-link` this opens a fresh round, enumerating everything
/// and ending with an `@odata.deltaLink`. Feed that link back through
/// `--delta-link` for the next round, which returns only what changed
/// since. A page printing a `nextPage` was truncated: that link goes back
/// through `--page-token` to drain the rest of the round.
///
/// JSON output: `{"contacts": [<raw Graph delta>...], "@odata.deltaLink",
/// "nextPage"}`, `nextPage` Graph's `@odata.nextLink`, `null` once the
/// round ends.
#[derive(Debug, Parser)]
pub struct MsgraphContactDeltaCommand {
    /// Contact folder id; omit for the default Contacts folder.
    #[arg(
        short = 'f',
        long,
        value_name = "FOLDER-ID",
        conflicts_with_all = ["delta_link", "page_token"]
    )]
    pub folder: Option<String>,
    /// Comma-separated properties to return (`$select`).
    #[arg(
        long,
        value_name = "CSV",
        conflicts_with_all = ["delta_link", "page_token"]
    )]
    pub select: Option<String>,
    /// Open the next round from the `@odata.deltaLink` a round ended with.
    ///
    /// The link already carries the folder and the properties, so neither
    /// is passed again.
    #[arg(long, value_name = "URL", conflicts_with = "page_token")]
    pub delta_link: Option<String>,
    #[command(flatten)]
    pub page_token: PageTokenArg,
}

impl MsgraphContactDeltaCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let link = self.page_token.token.as_ref().or(self.delta_link.as_ref());
        let page = match link {
            Some(link) => client.contacts_delta_from_link(link)?,
            None => client
                .contacts_delta(self.folder.as_deref(), self.select.as_deref())
                .map_err(folder_refusal)?,
        }
        .response;

        printer.out(Paginated {
            inner: MsgraphContactDeltaOutput {
                contacts: page.value,
                delta_link: page.delta_link,
            },
            next_page: page.next_link,
        })
    }
}

/// A page of the contacts delta, with the link opening the next round.
///
/// It prints through [`Paginated`], Graph's `@odata.nextLink`, which drains
/// the round, lifted into `nextPage`; the `@odata.deltaLink` ending the
/// round stays raw.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphContactDeltaOutput {
    /// The raw Graph contact deltas of this page.
    #[serde(rename = "contacts")]
    #[schemars(with = "Vec<serde_json::Value>")]
    pub contacts: Vec<MsgraphContactDelta>,
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

        if let Some(delta_link) = &self.delta_link {
            if !records.items.is_empty() {
                writeln!(f)?;
            }

            writeln!(f, "delta-link: {}", sanitize::text(delta_link))?;
        }

        Ok(())
    }
}
