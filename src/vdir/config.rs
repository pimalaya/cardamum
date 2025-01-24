use std::path::PathBuf;

use addressbook::vdir::Client;
use serde::{Deserialize, Serialize};
use shellexpand_utils::shellexpand_path;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct VdirConfig {
    pub home_dir: PathBuf,
}

impl From<VdirConfig> for Client {
    fn from(config: VdirConfig) -> Client {
        let home_dir = shellexpand_path(&config.home_dir);
        Client::new(home_dir)
    }
}
