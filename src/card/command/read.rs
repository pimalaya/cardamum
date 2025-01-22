use addressbook::Card;
use clap::Parser;
use color_eyre::Result;
use pimalaya_tui::terminal::{cli::printer::Printer, config::TomlConfig as _};

use crate::{
    account::{arg::name::AccountNameFlag, config::Backend},
    config::TomlConfig,
};

/// Read all folders.
///
/// This command allows you to read all exsting folders.
#[derive(Debug, Parser)]
pub struct ReadCardCommand {
    #[command(flatten)]
    pub account: AccountNameFlag,

    /// The identifier of the addressbook where the vCard should be
    /// read from.
    #[arg(name = "ADDRESSBOOK-ID")]
    pub addressbook_id: String,

    /// The identifier of the card that should be read.
    #[arg(name = "CARD-ID")]
    pub id: String,
}

impl ReadCardCommand {
    pub fn execute(self, printer: &mut impl Printer, config: TomlConfig) -> Result<()> {
        let (_, config) = config.to_toml_account_config(self.account.name.as_deref())?;

        let card: Card = match config.backend {
            Backend::None => {
                // SAFETY: case handled by the config deserializer
                unreachable!();
            }
            #[cfg(feature = "_carddav")]
            Backend::CardDav(config) => {
                use crate::carddav::Client;
                Client::new(config)?.read_card(self.addressbook_id, self.id)?
            }
        };

        printer.out(card.to_string().trim_end())
    }
}
