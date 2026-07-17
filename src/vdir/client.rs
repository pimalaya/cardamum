//! Cardamum wrapper around [`io_vdir::client::VdirClient`] that
//! bundles the merged [`Account`] alongside the vdir client.

use std::ops::{Deref, DerefMut};

use anyhow::{Result, anyhow};
use io_vdir::client::VdirClient as Inner;

use crate::{
    account::context::Account,
    config::{AccountConfig, Config, VdirConfig},
};

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

/// Builds the merged [`Account`] from the already-resolved config and
/// account, then opens the vdir client. Bails when the account has no
/// `[vdir]` block.
pub fn build_vdir_client(
    config: Config,
    name: String,
    mut account_config: AccountConfig,
) -> Result<VdirClient> {
    let vdir_config = account_config
        .vdir
        .take()
        .ok_or_else(|| anyhow!("Vdir config is missing for account `{name}`"))?;
    let account = Account::from(config).merge(Account::from(account_config));
    Ok(VdirClient::new(vdir_config, account))
}
