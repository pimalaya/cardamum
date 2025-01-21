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
pub struct DeleteAddressbookCommand {
    #[command(flatten)]
    pub account: AccountNameFlag,
    #[arg(name = "ID")]
    pub id: String,
    #[arg(long, short)]
    pub yes: bool,
}

impl DeleteAddressbookCommand {
    pub fn execute(self, printer: &mut impl Printer, config: TomlConfig) -> Result<()> {
        if !self.yes {
            let confirm = "Do you really want to delete this addressbook";
            let confirm = format!("{confirm}? All contacts will be definitely deleted.");

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
            #[cfg(any(
                feature = "carddav",
                feature = "carddav-native-tls",
                feature = "carddav-rustls",
            ))]
            Backend::CardDav(config) => {
                use crate::carddav::Client;
                Client::new(config)?.delete_addressbook(self.id)?
            }
        };

        printer.out("Addressbook successfully deleted")
    }
}
