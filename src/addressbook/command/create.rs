use addressbook::Addressbook;
use anyhow::Result;
use clap::Parser;
use pimalaya_tui::terminal::{cli::printer::Printer, config::TomlConfig as _};

use crate::{account::arg::name::AccountNameFlag, config::TomlConfig, Client};

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
        let client = Client::new(config.backend)?;

        let addressbook = Addressbook {
            name: self.name,
            desc: self.desc,
            color: self.color,
            ..Addressbook::default()
        };

        client.create_addressbook(addressbook)?;
        printer.out("Addressbook successfully created")
    }
}
