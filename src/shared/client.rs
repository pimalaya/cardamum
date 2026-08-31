//! # Addressbook client
//!
//! The cross-protocol client behind the shared addressbook and card
//! commands.
//!
//! It holds exactly one of the compiled-in backends and dispatches every
//! shared method to it, the glue itself living in the backend submodule
//! of each protocol module.

use anyhow::{Result, bail};
#[cfg(any(feature = "jmap", feature = "msgraph", feature = "people"))]
use pimalaya_config::secret::SecretResolver;

use crate::{
    account::context::Account,
    backend::Backend,
    config::{AccountConfig, Config},
    shared::{
        addressbook::{Addressbook, AddressbookDiff},
        card::{Card, CardUpdateOutcome},
    },
};

/// Cross-protocol addressbook client.
///
/// Bundles the active backend with the merged runtime [`Account`], which
/// carries the defaults a command falls back to.
pub struct AddressbookClient {
    inner: BackendClient,
    /// Runtime account the commands read their defaults from.
    pub account: Account,
}

/// The active backend of an [`AddressbookClient`].
enum BackendClient {
    #[cfg(feature = "vdir")]
    Vdir(crate::vdir::backend::VdirBackend),
    #[cfg(feature = "pimdir")]
    Pimdir(Box<crate::pimdir::backend::PimdirBackend>),
    #[cfg(feature = "carddav")]
    Carddav(Box<crate::carddav::backend::CarddavBackend>),
    #[cfg(feature = "jmap")]
    Jmap(Box<crate::jmap::backend::JmapBackend>),
    #[cfg(feature = "msgraph")]
    Msgraph(Box<crate::msgraph::backend::MsgraphBackend>),
    #[cfg(feature = "people")]
    People(Box<crate::people::backend::PeopleBackend>),
}

impl AddressbookClient {
    /// Builds the client from the account configuration.
    ///
    /// The first configured backend that `backend` allows wins, and the
    /// call bails when the account configures none of them.
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

        #[cfg(feature = "pimdir")]
        if inner.is_none()
            && backend.allows_pimdir()
            && let Some(pimdir_config) = account_config.pimdir.take()
        {
            use crate::pimdir::backend::PimdirBackend;
            let client = PimdirBackend::new(pimdir_config)?;
            inner = Some(BackendClient::Pimdir(Box::new(client)));
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
            let client = JmapBackend::new(jmap_config, &mut SecretResolver::new())?;
            inner = Some(BackendClient::Jmap(Box::new(client)));
        }

        #[cfg(feature = "msgraph")]
        if inner.is_none()
            && backend.allows_msgraph()
            && let Some(msgraph_config) = account_config.msgraph.take()
        {
            use crate::msgraph::backend::MsgraphBackend;
            let client = MsgraphBackend::new(msgraph_config, &mut SecretResolver::new())?;
            inner = Some(BackendClient::Msgraph(Box::new(client)));
        }

        #[cfg(feature = "people")]
        if inner.is_none()
            && backend.allows_people()
            && let Some(people_config) = account_config.people.take()
        {
            use crate::people::backend::PeopleBackend;
            let client = PeopleBackend::new(people_config, &mut SecretResolver::new())?;
            inner = Some(BackendClient::People(Box::new(client)));
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
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.list_addressbooks(),
            #[cfg(feature = "carddav")]
            BackendClient::Carddav(client) => client.list_addressbooks(),
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => client.list_addressbooks(),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.list_addressbooks(),
            #[cfg(feature = "people")]
            BackendClient::People(client) => client.list_addressbooks(),
        }
    }

    /// Creates an addressbook and returns the id the backend assigned.
    pub fn create_addressbook(
        &mut self,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<String> {
        match &mut self.inner {
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.create_addressbook(name, description, color),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.create_addressbook(name, description, color),
            #[cfg(feature = "carddav")]
            BackendClient::Carddav(client) => client.create_addressbook(name, description, color),
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => client.create_addressbook(name, description, color),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.create_addressbook(name, description, color),
            #[cfg(feature = "people")]
            BackendClient::People(client) => client.create_addressbook(name, description, color),
        }
    }

    /// Applies a partial update to the addressbook identified by `id`.
    pub fn update_addressbook(&mut self, id: &str, patch: AddressbookDiff) -> Result<()> {
        match &mut self.inner {
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.update_addressbook(id, patch),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.update_addressbook(id, patch),
            #[cfg(feature = "carddav")]
            BackendClient::Carddav(client) => client.update_addressbook(id, patch),
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => client.update_addressbook(id, patch),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.update_addressbook(id, patch),
            #[cfg(feature = "people")]
            BackendClient::People(client) => client.update_addressbook(id, patch),
        }
    }

    /// Deletes the addressbook identified by `id` and every card it
    /// exclusively contains.
    pub fn delete_addressbook(&mut self, id: &str) -> Result<()> {
        match &mut self.inner {
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.delete_addressbook(id),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.delete_addressbook(id),
            #[cfg(feature = "carddav")]
            BackendClient::Carddav(client) => client.delete_addressbook(id),
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => client.delete_addressbook(id),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.delete_addressbook(id),
            #[cfg(feature = "people")]
            BackendClient::People(client) => client.delete_addressbook(id),
        }
    }

    /// Lists a page of cards inside `addressbook_id`.
    ///
    /// The 1-indexed `page` defaults to the first one, and a `page_size`
    /// of `None` returns every card.
    pub fn list_cards(
        &mut self,
        addressbook_id: &str,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<Vec<Card>> {
        match &mut self.inner {
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.list_cards(addressbook_id, page, page_size),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.list_cards(addressbook_id, page, page_size),
            #[cfg(feature = "carddav")]
            BackendClient::Carddav(client) => client.list_cards(addressbook_id, page, page_size),
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => client.list_cards(addressbook_id, page, page_size),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.list_cards(addressbook_id, page, page_size),
            #[cfg(feature = "people")]
            BackendClient::People(client) => client.list_cards(addressbook_id, page, page_size),
        }
    }

    /// Fetches the card `card_id` from `addressbook_id`.
    pub fn get_card(&mut self, addressbook_id: &str, card_id: &str) -> Result<Card> {
        match &mut self.inner {
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.get_card(addressbook_id, card_id),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.get_card(addressbook_id, card_id),
            #[cfg(feature = "carddav")]
            BackendClient::Carddav(client) => client.get_card(addressbook_id, card_id),
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => client.get_card(addressbook_id, card_id),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.get_card(addressbook_id, card_id),
            #[cfg(feature = "people")]
            BackendClient::People(client) => client.get_card(addressbook_id, card_id),
        }
    }

    /// Appends a raw vCard and returns the id the backend assigned.
    pub fn create_card(&mut self, addressbook_id: &str, contents: Vec<u8>) -> Result<String> {
        match &mut self.inner {
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.create_card(addressbook_id, contents),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.create_card(addressbook_id, contents),
            #[cfg(feature = "carddav")]
            BackendClient::Carddav(client) => client.create_card(addressbook_id, contents),
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => client.create_card(addressbook_id, contents),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.create_card(addressbook_id, contents),
            #[cfg(feature = "people")]
            BackendClient::People(client) => client.create_card(addressbook_id, contents),
        }
    }

    /// Replaces the contents of `card_id` inside `addressbook_id`.
    ///
    /// The `if_match` entity tag gates the update, `None` overwriting
    /// unconditionally. A backend with no such guard either ignores the
    /// tag or bails, rather than pretending the check happened.
    pub fn update_card(
        &mut self,
        addressbook_id: &str,
        card_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
    ) -> Result<CardUpdateOutcome> {
        match &mut self.inner {
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => {
                client.update_card(addressbook_id, card_id, contents, if_match)
            }
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => {
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
            #[cfg(feature = "people")]
            BackendClient::People(client) => {
                client.update_card(addressbook_id, card_id, contents, if_match)
            }
        }
    }

    /// Permanently deletes `card_id` from `addressbook_id`.
    pub fn delete_card(&mut self, addressbook_id: &str, card_id: &str) -> Result<()> {
        match &mut self.inner {
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.delete_card(addressbook_id, card_id),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.delete_card(addressbook_id, card_id),
            #[cfg(feature = "carddav")]
            BackendClient::Carddav(client) => client.delete_card(addressbook_id, card_id),
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => client.delete_card(addressbook_id, card_id),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.delete_card(addressbook_id, card_id),
            #[cfg(feature = "people")]
            BackendClient::People(client) => client.delete_card(addressbook_id, card_id),
        }
    }
}

/// Applies 1-indexed pagination to an in-memory list.
///
/// A `page_size` of `None` returns every item, while a size of zero or a
/// page past the end returns nothing.
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
