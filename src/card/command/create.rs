use std::{
    env::{self, temp_dir},
    fs,
    process::{Command, Stdio},
};

use addressbook::Card;
use anyhow::{
    eyre::{bail, eyre},
    Result,
};
use clap::Parser;
use pimalaya_tui::terminal::{cli::printer::Printer, config::TomlConfig as _};

use crate::{account::arg::name::AccountNameFlag, config::TomlConfig, Client};

/// Create all folders.
///
/// This command allows you to create all exsting folders.
#[derive(Debug, Parser)]
pub struct CreateCardCommand {
    #[command(flatten)]
    pub account: AccountNameFlag,

    /// The identifier of the addressbook where the vCard should be
    /// added to.
    #[arg(name = "ADDRESSBOOK-ID")]
    pub addressbook_id: String,
}

impl CreateCardCommand {
    pub fn execute(self, printer: &mut impl Printer, config: TomlConfig) -> Result<()> {
        let (_, config) = config.to_toml_account_config(self.account.name.as_deref())?;
        let client = Client::new(config.backend)?;
        let uid = Card::generate_id();
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
        let card = Card::parse(Card::generate_id(), content).ok_or(eyre!("cannot parse vCard"))?;

        client.create_card(self.addressbook_id, card)?;

        printer.out("Card successfully created")
    }
}
