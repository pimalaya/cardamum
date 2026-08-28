use core::fmt;

use anyhow::Result;
use clap::Parser;
use comfy_table::{Cell, Color, Row, Table};
use io_msgraph::v1::rest::users::contacts::delta::MsgraphContactDelta;
use pimalaya_cli::printer::Printer;
use serde::Serialize;

use crate::{
    msgraph::{client::MsgraphClient, contacts::render::contact_name},
    shared::table::style_from_preset,
};

/// One page of the contacts delta (Graph incremental sync): the
/// contacts changed or removed since the last round.
///
/// Without `--delta-link` this opens a fresh round, enumerating
/// everything and ending with an `@odata.deltaLink`. Feed that link back
/// through `--delta-link` for the next round, which returns only what
/// changed since; a page ending with an `@odata.nextLink` was truncated,
/// and that link goes back through the same flag to drain the rest.
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
    /// `@odata.deltaLink` or `@odata.nextLink` from a previous page, to
    /// continue that round instead of opening a new one. The link
    /// already carries the folder and the properties, so neither is
    /// passed again.
    #[arg(long, value_name = "URL")]
    pub delta_link: Option<String>,
}

impl MsgraphContactDeltaCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let preset = client.account.table_preset().to_string();
        let id_color = client.account.cards_list_table_id_color();

        let page = match &self.delta_link {
            Some(link) => client.contacts_delta_from_link(link)?,
            None => client.contacts_delta(self.folder.as_deref(), self.select.as_deref())?,
        }
        .response;

        printer.out(DeltaReport {
            preset,
            id_color,
            contacts: page.value,
            next_link: page.next_link,
            delta_link: page.delta_link,
        })
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct DeltaReport {
    #[serde(skip)]
    pub preset: String,
    #[serde(skip)]
    pub id_color: Color,
    #[serde(rename = "contacts")]
    pub contacts: Vec<MsgraphContactDelta>,
    #[serde(rename = "@odata.nextLink", skip_serializing_if = "Option::is_none")]
    pub next_link: Option<String>,
    #[serde(rename = "@odata.deltaLink", skip_serializing_if = "Option::is_none")]
    pub delta_link: Option<String>,
}

impl fmt::Display for DeltaReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_style(style_from_preset(&self.preset))
            .set_header(Row::from([
                Cell::new("STATUS"),
                Cell::new("ID"),
                Cell::new("NAME"),
            ]))
            .add_rows(self.contacts.iter().map(|delta| {
                let status = if delta.removed.is_some() {
                    "removed"
                } else {
                    "changed"
                };
                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(Cell::new(status))
                    .add_cell(Cell::new(&delta.contact.id).fg(self.id_color))
                    .add_cell(Cell::new(contact_name(&delta.contact)));
                row
            }));

        writeln!(f)?;
        write!(f, "{table}")?;
        writeln!(f)?;
        if let Some(delta_link) = &self.delta_link {
            writeln!(f, "delta-link: {delta_link}")?;
        }
        if let Some(next_link) = &self.next_link {
            writeln!(f, "next-link: {next_link}")?;
            writeln!(f, "(page truncated: pass the next-link to --delta-link)")?;
        }
        Ok(())
    }
}
