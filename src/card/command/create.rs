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

/// Create a new card.
///
/// This command allows you to add a new vCard to the given
/// addressbook.
#[derive(Debug, Parser)]
pub struct CreateCardCommand {
    /// The identifier of the addressbook where the vCard should be
    /// added to.
    #[arg(name = "ADDRESSBOOK-ID")]
    pub addressbook_id: String,
}

impl CreateCardCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let mut client = Client::new(&account)?;

        let uid = Card::new_uuid();
        let path = temp_dir().join(format!("{uid}.vcf"));
        let tpl = format!(include_str!("./create.vcf"), uid);
        fs::write(&path, tpl)?;

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
            id: Card::new_uuid().to_string(),
            addressbook_id: self.addressbook_id,
            vcard: Card::parse(content).context("cannot parse vCard")?,
        };

        client.create_card(card)?;

        printer.out("Card successfully created")
    }
}
