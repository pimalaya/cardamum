//! # Vdir client
//!
//! Wraps [`io_vdir::client::VdirClient`] and carries the merged [`Account`]
//! alongside it, which is what the commands render from.

use std::{
    ops::{Deref, DerefMut},
    path::Path,
};

use anyhow::{Result, anyhow, bail};
use io_vdir::{client::VdirClient as Inner, path::VdirPath};

use crate::{
    account::context::Account,
    config::{AccountConfig, Config, VdirConfig},
};

/// The vdir client a command runs against, plus its account.
pub struct VdirClient {
    inner: Inner,
    pub account: Account,
}

impl VdirClient {
    /// Opens the client rooted at the configured home directory.
    pub fn new(config: VdirConfig, account: Account) -> Self {
        let inner = Inner::new(config.home_dir);
        Self { inner, account }
    }

    /// Resolves a collection name to its path under the vdir root.
    ///
    /// A missing directory is rejected here, io-vdir would otherwise
    /// surface a raw OS error.
    pub fn collection_path(&self, name: &str) -> Result<VdirPath> {
        let path = self.root().join(name);
        if !Path::new(path.as_str()).is_dir() {
            bail!("Collection `{name}` not found");
        }
        Ok(path)
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

/// Merges the global and account configs, then opens the vdir client.
///
/// Bails when the account declares no `[vdir]` block.
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
