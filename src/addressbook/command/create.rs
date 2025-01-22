use addressbook::Addressbook;
use clap::Parser;
use color_eyre::Result;
use pimalaya_tui::terminal::{cli::printer::Printer, config::TomlConfig as _};

use crate::{
    account::{arg::name::AccountNameFlag, config::Backend},
    config::TomlConfig,
};

/// Create all folders.
///
/// This command allows you to create all exsting folders.
#[derive(Debug, Parser)]
pub struct CreateAddressbookCommand {
    #[command(flatten)]
    pub account: AccountNameFlag,
    #[arg()]
    pub name: String,
    #[arg(long = "desc", short)]
    pub desc: Option<String>,
    #[arg(long, short = 'C')]
    pub color: Option<String>,
}

impl CreateAddressbookCommand {
    pub fn execute(self, printer: &mut impl Printer, config: TomlConfig) -> Result<()> {
        let (_, config) = config.to_toml_account_config(self.account.name.as_deref())?;

        let addressbook = Addressbook {
            name: self.name,
            desc: self.desc,
            color: self.color,
            ..Addressbook::default()
        };

        match config.backend {
            // SAFETY: case handled by the config deserializer
            Backend::None => unreachable!(),
            #[cfg(feature = "_carddav")]
            Backend::CardDav(config) => {
                use crate::carddav::Client;
                Client::new(config)?.create_addressbook(addressbook)?;
            }
        };

        printer.out("Addressbook successfully created")
    }
}
