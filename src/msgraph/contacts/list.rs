//! # Contact list command
//!
//! Lists one page of contacts of a folder.

use anyhow::Result;
use clap::Parser;
use io_msgraph::v1::rest::users::contacts::list::MsgraphContactsListParams;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{
    client::{MsgraphClient, folder_refusal},
    contacts::render::MsgraphContactsOutput,
};

/// List contacts of the default Contacts folder, or of `--folder` (one
/// Graph page).
///
/// JSON output: `{"contacts": [<raw Graph contact>...]}`.
#[derive(Debug, Parser)]
pub struct MsgraphContactListCommand {
    /// Contact folder id; omit for the default Contacts folder.
    #[arg(short = 'f', long, value_name = "FOLDER-ID")]
    pub folder: Option<String>,
    /// Maximum number of contacts in the page (`$top`).
    #[arg(short = 's', long, value_name = "N")]
    pub top: Option<u32>,
    /// Comma-separated properties to return (`$select`).
    #[arg(long, value_name = "CSV")]
    pub select: Option<String>,
    /// OData filter expression (`$filter`).
    #[arg(long, value_name = "EXPR")]
    pub filter: Option<String>,
}

impl MsgraphContactListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let params = MsgraphContactsListParams {
            top: self.top,
            select: self.select.as_deref(),
            filter: self.filter.as_deref(),
            ..Default::default()
        };
        let page = client
            .contacts_list(self.folder.as_deref(), &params)
            .map_err(folder_refusal)?
            .response;

        printer.out(MsgraphContactsOutput {
            contacts: page.value,
            next_link: page.next_link,
        })
    }
}
