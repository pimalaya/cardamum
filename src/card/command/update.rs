use std::{
    env::{self, temp_dir},
    fs,
    process::{Command, Stdio},
};

use addressbook::Card;
use clap::Parser;
use color_eyre::{
    eyre::{bail, eyre},
    Result,
};
use pimalaya_tui::terminal::{cli::printer::Printer, config::TomlConfig as _};

use crate::{account::arg::name::AccountNameFlag, config::TomlConfig, Client};

/// Update all folders.
///
/// This command allows you to update all exsting folders.
#[derive(Debug, Parser)]
pub struct UpdateCardCommand {
    #[command(flatten)]
    pub account: AccountNameFlag,

    /// The identifier of the addressbook where the vCard should be
    /// updated from.
    #[arg(name = "ADDRESSBOOK-ID")]
    pub addressbook_id: String,

    /// The identifier of the vCard to update.
    #[arg(name = "CARD-ID")]
    pub card_id: String,
}

impl UpdateCardCommand {
    pub fn execute(self, printer: &mut impl Printer, config: TomlConfig) -> Result<()> {
        let (_, config) = config.to_toml_account_config(self.account.name.as_deref())?;
        let client = Client::new(config.backend)?;

        let card = client.read_card(&self.addressbook_id, &self.card_id)?;

        let path = temp_dir().join(format!("{}.vcf", card.id));
        fs::write(&path, card.to_string())?;

        let args = env::var("EDITOR")?;
        let mut args = args.split_whitespace();
        let editor = args.next().unwrap();
        let edition = Command::new(editor)
            .args(args)
            .arg(&path)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;

        if !edition.success() {
            let code = edition.code();
            bail!("error while editing vCard: error code {code:?}");
        }

        let content = fs::read_to_string(&path)?
            .replace('\r', "")
            .replace('\n', "\r\n");
        let card = Card::parse(self.card_id, content).ok_or(eyre!("cannot parse vCard"))?;

        client.update_card(&self.addressbook_id, card)?;

        printer.out("Card successfully updated")
    }
}
