use addressbook::PartialAddressbook;
use anyhow::Result;
use clap::Parser;
use pimalaya_tui::terminal::{cli::printer::Printer, config::TomlConfig as _};

use crate::{account::arg::name::AccountNameFlag, config::TomlConfig, Client};

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
        let client = Client::new(config.backend)?;

        let addressbook = PartialAddressbook {
            id: self.id,
            name: self.name,
            desc: self.desc,
            color: self.color,
        };

        client.update_addressbook(addressbook)?;
        printer.out("Addressbook successfully updated")
    }
}
