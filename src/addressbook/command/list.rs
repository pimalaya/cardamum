use clap::Parser;
use color_eyre::Result;
use pimalaya_tui::terminal::{cli::printer::Printer, config::TomlConfig as _};

use crate::{
    account::arg::name::AccountNameFlag, addressbook::table::AddressbooksTable, config::TomlConfig,
    Client,
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
        let client = Client::new(config.backend)?;

        let addressbooks = client.list_addressbooks()?;
        let table = AddressbooksTable::from(addressbooks);
        printer.out(table)
    }
}
