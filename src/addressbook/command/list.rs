use addressbook::carddav::Client;
use clap::Parser;
use color_eyre::Result;
use pimalaya_tui::terminal::{cli::printer::Printer, config::TomlConfig as _};

use crate::{
    account::{arg::name::AccountNameFlag, config::Backend},
    addressbook::table::AddressbooksTable,
    config::TomlConfig,
};

/// List all folders.
///
/// This command allows you to list all exsting folders.
#[derive(Debug, Parser)]
pub struct ListAddressbooksCommand {
    #[command(flatten)]
    pub account: AccountNameFlag,
}

impl ListAddressbooksCommand {
    pub fn execute(self, printer: &mut impl Printer, config: TomlConfig) -> Result<()> {
        let (_, config) = config.to_toml_account_config(self.account.name.as_deref())?;

        let addressbooks = match config.backend {
            Backend::CardDav(config) => {
                let client = Client::try_from(config.clone())?;
                let mut flow = client.list_addressbooks();

                config.encryption.run(&client, &mut flow)?;
                flow.output()?
            }
        };

        let table = AddressbooksTable::from(addressbooks);
        printer.out(table)
    }
}
