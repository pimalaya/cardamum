use std::process;

use clap::Parser;
use color_eyre::Result;
use pimalaya_tui::terminal::{cli::printer::Printer, config::TomlConfig as _, prompt};

use crate::{
    account::{arg::name::AccountNameFlag, config::Backend},
    config::TomlConfig,
};

/// Delete all folders.
///
/// This command allows you to delete all exsting folders.
#[derive(Debug, Parser)]
pub struct DeleteCardCommand {
    #[command(flatten)]
    pub account: AccountNameFlag,

    /// The identifier of the addressbook where the vCard should be
    /// deleted from.
    #[arg(name = "ADDRESSBOOK-ID")]
    pub addressbook_id: String,

    /// The identifier of the vCard to delete.
    #[arg(name = "CARD-ID")]
    pub id: String,

    #[arg(long, short)]
    pub yes: bool,
}

impl DeleteCardCommand {
    pub fn execute(self, printer: &mut impl Printer, config: TomlConfig) -> Result<()> {
        if !self.yes {
            let confirm = "Do you really want to delete this card?";

            if !prompt::bool(confirm, false)? {
                process::exit(0);
            };
        };

        let (_, config) = config.to_toml_account_config(self.account.name.as_deref())?;

        match config.backend {
            Backend::None => {
                // SAFETY: case handled by the config deserializer
                unreachable!();
            }
            #[cfg(feature = "_carddav")]
            Backend::CardDav(config) => {
                use crate::carddav::Client;
                Client::new(config)?.delete_card(self.addressbook_id, self.id)?
            }
        };

        printer.out("Card successfully deleted")
    }
}
