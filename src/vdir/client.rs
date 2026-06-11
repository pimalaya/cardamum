//! Cardamum wrapper around [`io_vdir::client::VdirClient`] that
//! bundles the merged [`Account`] alongside the vdir client.

use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use anyhow::{Result, anyhow};
use io_vdir::client::VdirClient as Inner;
use pimalaya_config::toml::TomlConfig;

use crate::{account::context::Account, cli::load_or_wizard, config::VdirConfig};

pub struct VdirClient {
    inner: Inner,
    pub account: Account,
}

impl VdirClient {
    pub fn new(config: VdirConfig, account: Account) -> Self {
        let inner = Inner::new(config.home_dir);
        Self { inner, account }
    }
}

impl Deref for VdirClient {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for VdirClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Loads the configuration, picks the active account, builds the
/// merged [`Account`] then opens the vdir client. Bails when the
/// account has no `[vdir]` block.
pub fn build_vdir_client(
    config_paths: &[PathBuf],
    account_name: Option<&str>,
) -> Result<VdirClient> {
    let mut config = load_or_wizard(config_paths)?;
    let (name, mut ac) = config
        .take_account(account_name)?
        .ok_or_else(|| anyhow!("Cannot find account"))?;
    let vdir_config = ac
        .vdir
        .take()
        .ok_or_else(|| anyhow!("Vdir config is missing for account `{name}`"))?;
    let account = Account::from(config).merge(Account::from(ac));
    Ok(VdirClient::new(vdir_config, account))
}
