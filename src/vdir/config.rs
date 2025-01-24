use std::path::{Path, PathBuf};

use addressbook::vdir::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct VdirConfig {
    pub home_dir: PathBuf,
}

impl VdirConfig {
    pub fn home_dir(&self) -> &Path {
        self.home_dir.as_ref()
    }
}

impl From<VdirConfig> for Client {
    fn from(config: VdirConfig) -> Client {
        Client::new(config.home_dir)
    }
}
