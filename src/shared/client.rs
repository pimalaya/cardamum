//! Cross-protocol [`AddressbookClient`] for the shared subcommands
//! (`addressbooks`, `cards`).
//!
//! One variant per compiled-in backend (vdir, CardDAV, JMAP, Microsoft
//! Graph, Google People); a value always holds exactly one. Each
//! shared-API method dispatches to the active backend's matching
//! method; the per-backend glue lives in each protocol module's
//! backend submodule.

use anyhow::{Result, bail};

use crate::{
    account::context::Account,
    backend::Backend,
    config::{AccountConfig, Config},
    shared::{
        addressbook::{Addressbook, AddressbookDiff},
        card::Card,
    },
};

/// Cross-protocol addressbook client bundling the active backend and
/// the merged runtime [`Account`].
pub struct AddressbookClient {
    inner: BackendClient,
    pub account: Account,
}

/// The active backend of an [`AddressbookClient`]: exactly one of the
/// compiled-in per-backend glue clients.
enum BackendClient {
    #[cfg(feature = "vdir")]
    Vdir(crate::vdir::backend::VdirBackend),
    #[cfg(feature = "carddav")]
    Carddav(Box<crate::carddav::backend::CarddavBackend>),
    #[cfg(feature = "jmap")]
    Jmap(Box<crate::jmap::backend::JmapBackend>),
    #[cfg(feature = "msgraph")]
    Msgraph(Box<crate::msgraph::backend::MsgraphBackend>),
    #[cfg(feature = "google")]
    Google(Box<crate::google::backend::GoogleBackend>),
}

impl AddressbookClient {
    /// Builds the client from the account configuration: the first
    /// configured backend allowed by `backend` wins.
    pub fn new(
        config: Config,
        #[allow(unused_mut)] mut account_config: AccountConfig,
        backend: Backend,
    ) -> Result<Self> {
        #[allow(unused_mut)]
        let mut inner: Option<BackendClient> = None;

        #[cfg(feature = "vdir")]
        if inner.is_none()
            && backend.allows_vdir()
            && let Some(vdir_config) = account_config.vdir.take()
        {
            use crate::vdir::backend::VdirBackend;
            inner = Some(BackendClient::Vdir(VdirBackend::new(vdir_config)));
        }

        #[cfg(feature = "carddav")]
        if inner.is_none()
            && backend.allows_carddav()
            && let Some(carddav_config) = account_config.carddav.take()
        {
            use crate::carddav::backend::CarddavBackend;
            let client = CarddavBackend::new(carddav_config)?;
            inner = Some(BackendClient::Carddav(Box::new(client)));
        }

        #[cfg(feature = "jmap")]
        if inner.is_none()
            && backend.allows_jmap()
            && let Some(jmap_config) = account_config.jmap.take()
        {
            use crate::jmap::backend::JmapBackend;
            let client = JmapBackend::new(jmap_config)?;
            inner = Some(BackendClient::Jmap(Box::new(client)));
        }

        #[cfg(feature = "msgraph")]
        if inner.is_none()
            && backend.allows_msgraph()
            && let Some(msgraph_config) = account_config.msgraph.take()
        {
            use crate::msgraph::backend::MsgraphBackend;
            let client = MsgraphBackend::new(msgraph_config)?;
            inner = Some(BackendClient::Msgraph(Box::new(client)));
        }

        #[cfg(feature = "google")]
        if inner.is_none()
            && backend.allows_google()
            && let Some(google_config) = account_config.google.take()
        {
            use crate::google::backend::GoogleBackend;
            let client = GoogleBackend::new(google_config)?;
            inner = Some(BackendClient::Google(Box::new(client)));
        }

        let Some(inner) = inner else {
            bail!("No backend matching `{backend}` is configured for this account");
        };

        let account = Account::from(config).merge(Account::from(account_config));

        Ok(Self { inner, account })
    }

    /// Lists every addressbook available to the active account.
    pub fn list_addressbooks(&mut self) -> Result<Vec<Addressbook>> {
        match &mut self.inner {
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.list_addressbooks(),
            #[cfg(feature = "carddav")]
            BackendClient::Carddav(client) => client.list_addressbooks(),
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => client.list_addressbooks(),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.list_addressbooks(),
            #[cfg(feature = "google")]
            BackendClient::Google(client) => client.list_addressbooks(),
        }
    }

    /// Creates an addressbook named `name`, optionally carrying a
    /// description and a color. Returns the backend-assigned id.
    pub fn create_addressbook(
        &mut self,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<String> {
        match &mut self.inner {
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.create_addressbook(name, description, color),
            #[cfg(feature = "carddav")]
            BackendClient::Carddav(client) => client.create_addressbook(name, description, color),
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => client.create_addressbook(name, description, color),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.create_addressbook(name, description, color),
            #[cfg(feature = "google")]
            BackendClient::Google(client) => client.create_addressbook(name, description, color),
        }
    }

    /// Applies a partial update to the addressbook identified by `id`.
    /// Fields left as `None` in `patch` are preserved.
    pub fn update_addressbook(&mut self, id: &str, patch: AddressbookDiff) -> Result<()> {
        match &mut self.inner {
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.update_addressbook(id, patch),
            #[cfg(feature = "carddav")]
            BackendClient::Carddav(client) => client.update_addressbook(id, patch),
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => client.update_addressbook(id, patch),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.update_addressbook(id, patch),
            #[cfg(feature = "google")]
            BackendClient::Google(client) => client.update_addressbook(id, patch),
        }
    }

    /// Deletes the addressbook identified by `id` and every card it
    /// exclusively contains.
    pub fn delete_addressbook(&mut self, id: &str) -> Result<()> {
        match &mut self.inner {
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.delete_addressbook(id),
            #[cfg(feature = "carddav")]
            BackendClient::Carddav(client) => client.delete_addressbook(id),
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => client.delete_addressbook(id),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.delete_addressbook(id),
            #[cfg(feature = "google")]
            BackendClient::Google(client) => client.delete_addressbook(id),
        }
    }

    /// Lists cards inside `addressbook_id`. `page` is 1-indexed; pass
    /// `None` to default to page 1. `page_size = None` returns the full
    /// window.
    pub fn list_cards(
        &mut self,
        addressbook_id: &str,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<Vec<Card>> {
        match &mut self.inner {
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.list_cards(addressbook_id, page, page_size),
            #[cfg(feature = "carddav")]
            BackendClient::Carddav(client) => client.list_cards(addressbook_id, page, page_size),
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => client.list_cards(addressbook_id, page, page_size),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.list_cards(addressbook_id, page, page_size),
            #[cfg(feature = "google")]
            BackendClient::Google(client) => client.list_cards(addressbook_id, page, page_size),
        }
    }

    /// Fetches the card `card_id` from `addressbook_id`.
    pub fn get_card(&mut self, addressbook_id: &str, card_id: &str) -> Result<Card> {
        match &mut self.inner {
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.get_card(addressbook_id, card_id),
            #[cfg(feature = "carddav")]
            BackendClient::Carddav(client) => client.get_card(addressbook_id, card_id),
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => client.get_card(addressbook_id, card_id),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.get_card(addressbook_id, card_id),
            #[cfg(feature = "google")]
            BackendClient::Google(client) => client.get_card(addressbook_id, card_id),
        }
    }

    /// Appends a raw vCard to `addressbook_id`. Returns the identifier
    /// the backend assigned to the stored card.
    pub fn create_card(&mut self, addressbook_id: &str, contents: Vec<u8>) -> Result<String> {
        match &mut self.inner {
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.create_card(addressbook_id, contents),
            #[cfg(feature = "carddav")]
            BackendClient::Carddav(client) => client.create_card(addressbook_id, contents),
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => client.create_card(addressbook_id, contents),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.create_card(addressbook_id, contents),
            #[cfg(feature = "google")]
            BackendClient::Google(client) => client.create_card(addressbook_id, contents),
        }
    }

    /// Replaces the contents of `card_id` inside `addressbook_id`.
    ///
    /// `if_match` is the entity tag to gate the update on; pass `None`
    /// to overwrite unconditionally. Backends without a guard concept
    /// either ignore it (vdir) or bail (JMAP, Microsoft Graph).
    pub fn update_card(
        &mut self,
        addressbook_id: &str,
        card_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
    ) -> Result<()> {
        match &mut self.inner {
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => {
                client.update_card(addressbook_id, card_id, contents, if_match)
            }
            #[cfg(feature = "carddav")]
            BackendClient::Carddav(client) => {
                client.update_card(addressbook_id, card_id, contents, if_match)
            }
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => {
                client.update_card(addressbook_id, card_id, contents, if_match)
            }
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => {
                client.update_card(addressbook_id, card_id, contents, if_match)
            }
            #[cfg(feature = "google")]
            BackendClient::Google(client) => {
                client.update_card(addressbook_id, card_id, contents, if_match)
            }
        }
    }

    /// Permanently deletes `card_id` from `addressbook_id`.
    pub fn delete_card(&mut self, addressbook_id: &str, card_id: &str) -> Result<()> {
        match &mut self.inner {
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.delete_card(addressbook_id, card_id),
            #[cfg(feature = "carddav")]
            BackendClient::Carddav(client) => client.delete_card(addressbook_id, card_id),
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => client.delete_card(addressbook_id, card_id),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.delete_card(addressbook_id, card_id),
            #[cfg(feature = "google")]
            BackendClient::Google(client) => client.delete_card(addressbook_id, card_id),
        }
    }
}

/// 1-indexed pagination on an in-memory list. `page_size = None`
/// returns the full slice; `page_size = 0` or a page past the end
/// returns an empty vector.
pub fn paginate<T>(items: Vec<T>, page: Option<u32>, page_size: Option<u32>) -> Vec<T> {
    let Some(size) = page_size else {
        return items;
    };

    if size == 0 {
        return Vec::new();
    }

    let page = page.unwrap_or(1).max(1);
    let skip = ((page - 1) as usize).saturating_mul(size as usize);

    if skip >= items.len() {
        return Vec::new();
    }

    items.into_iter().skip(skip).take(size as usize).collect()
}
