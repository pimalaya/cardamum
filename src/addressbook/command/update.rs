use addressbook::PartialAddressbook;
use clap::Parser;
use color_eyre::Result;
use pimalaya_tui::terminal::{cli::printer::Printer, config::TomlConfig as _};

use crate::{
    account::{arg::name::AccountNameFlag, config::Backend},
    config::TomlConfig,
};

/// Update all folders.
///
/// This command allows you to update all exsting folders.
#[derive(Debug, Parser)]
pub struct UpdateAddressbookCommand {
    #[command(flatten)]
    pub account: AccountNameFlag,
    #[arg()]
    pub id: String,
    #[arg(long, short)]
    pub name: Option<String>,
    #[arg(long, short = 'C')]
    pub color: Option<String>,
    #[arg(long = "desc", short)]
    pub desc: Option<String>,
}

impl UpdateAddressbookCommand {
    pub fn execute(self, printer: &mut impl Printer, config: TomlConfig) -> Result<()> {
        let (_, config) = config.to_toml_account_config(self.account.name.as_deref())?;

        let addressbook = PartialAddressbook {
            id: self.id,
            name: self.name,
            desc: self.desc,
            color: self.color,
        };

        match config.backend {
            // SAFETY: case handled by the config deserializer
            Backend::None => unreachable!(),
            #[cfg(any(
                feature = "carddav",
                feature = "carddav-native-tls",
                feature = "carddav-rustls",
            ))]
            Backend::CardDav(config) => {
                use crate::carddav::Client;
                Client::new(config)?.update_addressbook(addressbook)?;
            }
        };

        printer.out("Addressbook successfully updated")
    }
}
