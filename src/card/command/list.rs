use std::process::Command;

use clap::Parser;
use color_eyre::{eyre::bail, Result};
use pimalaya_tui::terminal::{cli::printer::Printer, config::TomlConfig as _};

use crate::{
    account::{arg::name::AccountNameFlag, config::Backend},
    carddav::sans_io::ListCardsFlow,
    config::TomlConfig,
    contact::{Authentication, Cards, CardsTable, Encryption},
    tcp::{sans_io::Io as TcpIo, std::Connector},
    tls::std::RustlsConnector,
};

/// List all folders.
///
/// This command allows you to list all exsting folders.
#[derive(Debug, Parser)]
pub struct ListCardsCommand {
    #[command(flatten)]
    pub account: AccountNameFlag,

    /// The identifier of the CardDAV addressbook to list vCards from.
    #[arg(name = "ADDRESSBOOK-ID")]
    pub id: String,
}

impl ListCardsCommand {
    pub fn execute(self, printer: &mut impl Printer, config: TomlConfig) -> Result<()> {
        let (_, toml_account_config) =
            config.to_toml_account_config(self.account.name.as_deref())?;
        let (_, backend) = toml_account_config.into();

        let cards = match backend {
            Backend::None => bail!("cannot list cards: backend is not defined"),
            Backend::CardDav(config) => match config.authentication {
                Authentication::None => unimplemented!(),
                Authentication::Basic(auth) => {
                    let mut args = auth.password.split_whitespace();
                    let program = args.next().unwrap();
                    let password = Command::new(program).args(args).output().unwrap().stdout;
                    let password = String::from_utf8_lossy(password.trim_ascii());
                    let mut flow = ListCardsFlow::new(
                        &self.id,
                        &config.http_version,
                        &auth.username,
                        &password,
                    );

                    match config.encryption {
                        Encryption::None => {
                            let mut tcp = Connector::connect(&config.hostname, config.port)?;

                            while let Some(io) = flow.next() {
                                match io {
                                    TcpIo::Read => {
                                        tcp.read(&mut flow)?;
                                    }
                                    TcpIo::Write => {
                                        tcp.write(&mut flow)?;
                                    }
                                }
                            }
                        }
                        Encryption::Rustls(_) => {
                            let mut tls = RustlsConnector::connect(&config.hostname, config.port)?;

                            while let Some(io) = flow.next() {
                                match io {
                                    TcpIo::Read => {
                                        tls.read(&mut flow)?;
                                    }
                                    TcpIo::Write => {
                                        tls.write(&mut flow)?;
                                    }
                                }
                            }
                        }
                    }

                    Cards::try_from(flow)?
                }
            },
        };

        let table = CardsTable::from(cards);
        printer.out(table)
    }
}
