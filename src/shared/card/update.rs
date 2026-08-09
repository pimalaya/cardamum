use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::{arg::AddressbookIdArg, card::vcard::VcardArg, client::AddressbookClient};

/// Replace the bytes of an existing vCard.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct CardUpdateCommand {
    #[command(flatten)]
    pub addressbook: AddressbookIdArg,
    /// ETag returned by the previous read; when set, the update is
    /// gated on a server-side match (RFC 9110 If-Match).
    #[arg(long, value_name = "ETAG")]
    pub if_match: Option<String>,

    /// Card UID.
    #[arg(value_name = "CARD-ID")]
    pub card_id: String,
    #[command(flatten)]
    pub vcard: VcardArg,
}

impl CardUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: AddressbookClient) -> Result<()> {
        let addressbook_id = client.account.addressbook_id(self.addressbook.id)?;
        let contents = self.vcard.read()?;

        let outcome = client.update_card(
            &addressbook_id,
            &self.card_id,
            contents,
            self.if_match.as_deref(),
        )?;

        let mut msg = format!("Card `{}` successfully updated", self.card_id);

        // NOTE: the write landed, but not all of it: saying so beats a
        // clean success for a card that kept what the vCard dropped.
        if !outcome.kept_properties.is_empty() {
            msg.push_str(&format!(
                ", except {}, which the server will not let go",
                outcome.kept_properties.join(", ")
            ));
        }

        printer.out(Message::new(msg))
    }
}
