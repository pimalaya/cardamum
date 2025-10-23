use std::{
    env::{self, temp_dir},
    fs,
    process::{Command, Stdio},
};

use anyhow::{bail, Context, Result};
use clap::Parser;
use io_addressbook::card::Card;
use pimalaya_toolbox::terminal::printer::Printer;

use crate::{account::Account, client::Client};

/// Update a card.
///
/// This command allows you to update a vCard from an addressbook.
#[derive(Debug, Parser)]
pub struct UpdateCardCommand {
    /// The identifier of the addressbook where the vCard should be
    /// updated from.
    #[arg(name = "ADDRESSBOOK-ID")]
    pub addressbook_id: String,

    /// The identifier of the vCard to update.
    #[arg(name = "CARD-ID")]
    pub card_id: String,
}

impl UpdateCardCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let mut client = Client::new(&account)?;

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

        let card = Card {
            id: self.card_id,
            addressbook_id: self.addressbook_id,
            vcard: Card::parse(content).context("cannot parse vCard")?,
        };

        println!("pre update");
        client.update_card(card)?;

        printer.out("Card successfully updated")
    }
}
