use anyhow::{Result, bail};
use clap::Parser;
use io_jmap::rfc9610::contact_card::set::JmapContactCardSetArgs;
use pimalaya_cli::printer::{Message, Printer};

use crate::jmap::client::JmapClient;

/// Destroy a ContactCard (`ContactCard/set` destroy).
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct JmapContactCardDestroyCommand {
    /// ContactCard id.
    #[arg(value_name = "CARD-ID")]
    pub id: String,
}

impl JmapContactCardDestroyCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: JmapClient) -> Result<()> {
        let args = JmapContactCardSetArgs {
            destroy: Some(vec![self.id.clone()]),
            ..Default::default()
        };
        let out = client.contact_card_set(args)?;

        if let Some(err) = out.not_destroyed.into_values().next() {
            bail!("ContactCard destroy rejected: {err:?}");
        }

        printer.out(Message::new(format!(
            "ContactCard `{}` successfully destroyed",
            self.id
        )))
    }
}
