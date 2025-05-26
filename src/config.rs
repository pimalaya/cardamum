use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::account::config::TomlAccountConfig;

/// The main configuration.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TomlConfig {
    /// The configuration of all the accounts.
    pub accounts: HashMap<String, TomlAccountConfig>,
}

// #[cfg(feature = "wizard")]
// impl TomlConfig {
//     pub fn from_wizard(path: &std::path::Path) -> anyhow::Result<Self> {
//         crate::wizard::edit(path, Self::default(), None, Default::default())
//     }
// }

impl pimalaya_tui::terminal::config::TomlConfig for TomlConfig {
    type TomlAccountConfig = TomlAccountConfig;

    fn project_name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn get_default_account_config(&self) -> Option<(String, Self::TomlAccountConfig)> {
        for (name, account) in &self.accounts {
            if account.default {
                return Some((name.clone(), account.clone()));
            }
        }

        None
    }

    fn get_account_config(&self, name: &str) -> Option<(String, Self::TomlAccountConfig)> {
        self.accounts
            .get(name)
            .map(|account| (name.to_owned(), account.clone()))
    }

    // fn from_paths_or_default(paths: &[PathBuf]) -> pimalaya_tui::Result<Self> {
    //     match paths.len() {
    //         0 => Self::from_default_paths(),
    //         _ if paths[0].exists() => Self::from_paths(paths),
    //         #[cfg(feature = "wizard")]
    //         _ => {
    //             use pimalaya_tui::terminal::{print, prompt};

    //             let path = &paths[0];
    //             print::warn(format!("Cannot find configuration at {}.", path.display()));

    //             if !prompt::bool("Would you like to create one with the wizard?", true)? {
    //                 std::process::exit(0);
    //             }

    //             Self::from_wizard(&paths[0])
    //                 .map_err(pimalaya_tui::Error::CreateTomlConfigFromWizardError)
    //         }
    //         #[cfg(not(feature = "wizard"))]
    //         _ => Err(pimalaya_tui::Error::CreateTomlConfigFromInvalidPathsError),
    //     }
    // }
}
