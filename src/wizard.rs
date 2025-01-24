use std::{fmt, path::Path};

use color_eyre::Result;
use pimalaya_tui::terminal::{print, prompt};

use crate::{
    account::config::{Backend, TomlAccountConfig},
    config::TomlConfig,
};

#[derive(Eq, PartialEq)]
pub enum BackendKind {
    CardDav,
    Vdir,
}

impl fmt::Display for BackendKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CardDav => write!(f, "CardDAV"),
            Self::Vdir => write!(f, "Vdir"),
        }
    }
}

impl BackendKind {
    pub const ALL: [Self; 2] = [Self::CardDav, Self::Vdir];
}

pub fn edit(
    path: impl AsRef<Path>,
    mut config: TomlConfig,
    account_name: Option<&str>,
    mut account_config: TomlAccountConfig,
) -> Result<TomlConfig> {
    todo!()

    // match account_name.as_ref() {
    //     Some(name) => print::section(format!("Configuring your account {name}")),
    //     None => print::section("Configuring your default account"),
    // };

    // let default =
    //     account_name.is_none() || prompt::bool("Should this account be the default one?", false)?;

    // if default {
    //     config
    //         .accounts
    //         .iter_mut()
    //         .for_each(|(_, config)| config.default = false)
    // }

    // account_config.default = default;

    // let account_name = prompt::text("Account name:", None)?;

    // let backend = prompt::item("Backend:", &BackendKind::ALL, None)?;

    // match backend {
    //     #[cfg(feature = "_carddav")]
    //     BackendKind::CardDav => {
    //         account_config.backend = Backend::CardDav(config);
    //     }
    //     #[cfg(not(feature = "_carddav"))]
    //     BackendKind::CardDav => {
    //         bail!("Missing cargo feature `carddav`, `carddav-native-tls` or `carddav-rustls`");
    //     }
    //     #[cfg(feature = "_vdir")]
    //     BackendKind::Vdir => {
    //         account_config.backend = Backend::Vdir(config);
    //     }
    //     #[cfg(not(feature = "_vdir"))]
    //     BackendKind::Vdir => {
    //         bail!("Missing cargo feature `vdir`");
    //     }
    // }

    // config.accounts.insert(account_name, account_config);
    // config.write(path.as_ref())?;

    // Ok(config)
}
