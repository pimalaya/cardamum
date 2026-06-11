//! Cross-protocol [`AddressbookClient`] for the shared subcommands
//! (`addressbooks`, `cards`).
//!
//! Wraps [`io_addressbook::client::AddressbookClientStd`] and bundles
//! the active [`Account`] alongside the I/O client. Implements
//! [`Deref`]/[`DerefMut`] onto the inner client so callers can call
//! its methods directly.

use core::ops::{Deref, DerefMut};

use anyhow::{Result, bail};
use io_addressbook::client::AddressbookClientStd;

use crate::{
    account::context::Account,
    backend::Backend,
    config::{AccountConfig, Config},
};

pub struct AddressbookClient {
    inner: AddressbookClientStd,
    pub account: Account,
}

impl AddressbookClient {
    pub fn new(
        config: Config,
        #[allow(unused_mut)] mut account_config: AccountConfig,
        backend: Backend,
    ) -> Result<Self> {
        // The unified client is an enum holding exactly one backend, so
        // the first configured-and-allowed backend wins and is wrapped
        // via `From`.
        #[allow(unused_mut)]
        let mut inner: Option<AddressbookClientStd> = None;

        #[cfg(feature = "vdir")]
        if inner.is_none() && backend.allows_vdir() {
            if let Some(vdir_config) = account_config.vdir.take() {
                use io_addressbook::vdir::client::VdirClient;
                use io_vdir::client::VdirClient as InnerVdirClient;

                let client = VdirClient::new(InnerVdirClient::new(vdir_config.home_dir));
                inner = Some(AddressbookClientStd::from(client));
            }
        }

        #[cfg(feature = "carddav")]
        if inner.is_none() && backend.allows_carddav() {
            if let Some(carddav_config) = account_config.carddav.take() {
                use io_addressbook::webdav::client::WebdavClient;

                let connected = crate::carddav::client::open_carddav_client(carddav_config)?;
                inner = Some(AddressbookClientStd::from(WebdavClient::new(connected)));
            }
        }

        let Some(inner) = inner else {
            bail!("No backend matching `{backend}` is configured for this account");
        };

        let account = Account::from(config).merge(Account::from(account_config));

        Ok(Self { inner, account })
    }
}

impl Deref for AddressbookClient {
    type Target = AddressbookClientStd;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for AddressbookClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
