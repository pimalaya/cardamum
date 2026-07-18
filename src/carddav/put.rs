use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::{carddav::client::CarddavClient, shared::card::vcard::VcardArg};

/// PUT a card: create or replace its raw vCard bytes.
///
/// A plain PUT creates or overwrites unconditionally. `--if-none-match
/// '*'` makes it a create-only PUT (fails if the resource exists);
/// `--if-match <ETAG>` gates the replace on the resource being unchanged
/// (RFC 9110). JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct CarddavPutCommand {
    /// Identifier of the parent addressbook.
    #[arg(value_name = "ADDRESSBOOK")]
    pub addressbook_id: String,
    /// Card resource id (its href last path segment).
    #[arg(value_name = "CARD-ID")]
    pub card_id: String,
    #[command(flatten)]
    pub vcard: VcardArg,
    /// Gate the replace on the resource being unchanged (If-Match).
    #[arg(long, value_name = "ETAG", conflicts_with = "if_none_match")]
    pub if_match: Option<String>,
    /// Create-only PUT: fail if the resource already exists (pass `*`).
    #[arg(long, value_name = "VALUE", conflicts_with = "if_match")]
    pub if_none_match: Option<String>,
}

impl CarddavPutCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CarddavClient) -> Result<()> {
        let contents = self.vcard.read()?;

        let etag = if self.if_none_match.is_some() {
            client
                .create_card(&self.addressbook_id, &self.card_id, contents)?
                .etag
        } else {
            client
                .update_card(
                    &self.addressbook_id,
                    &self.card_id,
                    contents,
                    self.if_match.as_deref(),
                )?
                .etag
        };

        let etag = etag.unwrap_or_else(|| "(none)".into());
        printer.out(Message::new(format!(
            "Card `{}` successfully put (etag {etag})",
            self.card_id
        )))
    }
}
