use std::{
    env::{self, temp_dir},
    fs,
    process::{Command, Stdio},
};

use addressbook::Card;
use clap::Parser;
use color_eyre::{eyre::bail, Result};
use pimalaya_tui::terminal::{cli::printer::Printer, config::TomlConfig as _};
use uuid::Uuid;

use crate::{
    account::{arg::name::AccountNameFlag, config::Backend},
    config::TomlConfig,
};

/// Create all folders.
///
/// This command allows you to create all exsting folders.
#[derive(Debug, Parser)]
pub struct CreateCardCommand {
    #[command(flatten)]
    pub account: AccountNameFlag,

    /// The addressbook identifier where the vCard should be added to.
    #[arg(name = "ADDRESSBOOK-ID")]
    pub addressbook_id: String,
}

impl CreateCardCommand {
    pub fn execute(self, printer: &mut impl Printer, config: TomlConfig) -> Result<()> {
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

                let uid = Uuid::new_v4();
                let path = temp_dir().join(format!("{uid:x}.vcf"));
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

                let mut card = Card::default();
                card.content = fs::read_to_string(&path)?;

                Client::new(config)?.create_card(self.addressbook_id, card)?
            }
        };

        printer.out("Card successfully created")
    }
}
