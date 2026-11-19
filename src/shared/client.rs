//! # Addressbook client
//!
//! The cross-protocol client behind the unified addressbook and card
//! commands.
//!
//! It holds exactly one of the compiled-in backends and dispatches every
//! unified method to it, the glue itself living in the backend submodule
//! of each protocol module. A build without any backend keeps the client,
//! the unified command tree being the same in every build, with no arm to
//! read its arguments.

#![cfg_attr(
    not(any(
        feature = "vdir",
        feature = "pimdir",
        feature = "carddav",
        feature = "jmap",
        feature = "msgraph",
        feature = "gpeople"
    )),
    allow(dead_code, unused_variables)
)]

#[cfg(not(any(
    feature = "vdir",
    feature = "pimdir",
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
)))]
use std::convert::Infallible;

use anyhow::{Error, Result, anyhow, bail};
use log::debug;
use pimalaya_config::secret::SecretResolver;
use vcard::tree::cst::VcardCst;

use crate::{
    account::context::Account,
    backend::Backend,
    capability::{self, Capability},
    config::{AccountConfig, Config},
    error::{self, ItemError},
    shared::{
        addressbook::{Addressbook, AddressbookDiff},
        card::{Card, CardUpdateOutcome},
    },
};

/// Cross-protocol addressbook client.
///
/// Bundles the active backend with the merged runtime [`Account`], which
/// carries the defaults a command falls back to.
///
/// The connection is opened on the first call that needs it and can be
/// dropped again with [`disconnect`](Self::disconnect). A command running
/// a composer holds none open while the editor is up: a server closes an
/// idle connection, and the write landing after a long edit would read
/// the end of a socket nobody is on the other end of any more.
pub struct AddressbookClient {
    /// What it takes to open the backend, kept so it can be reopened.
    config: BackendConfig,
    /// The open backend, `None` until something needs it.
    inner: Option<BackendClient>,
    /// The credentials resolved so far, kept so a reopened backend
    /// spawns no secret command twice.
    resolver: SecretResolver,
    /// The name of the account.
    pub name: String,
    /// Runtime account the commands read their defaults from.
    pub account: Account,
}

/// The backend the unified commands run against: the first one the account
/// configures that `backend` allows, in priority order.
///
/// Fails coded `backend-not-built` when, under `auto`, that first one is
/// a backend the build leaves out: the account's backend is the same in
/// every build, so another build never silently takes the next one.
pub fn resolve_backend(
    account_config: &AccountConfig,
    backend: Backend,
) -> Result<Option<Backend>> {
    for name in account_config.backends() {
        match name.parse::<Backend>() {
            Ok(found) if backend == Backend::Auto || backend == found => return Ok(Some(found)),
            Err(_) if backend == Backend::Auto => return Err(crate::backend::not_built(name)),
            _ => continue,
        }
    }

    Ok(None)
}

/// The configuration of the backend an [`AddressbookClient`] speaks to.
enum BackendConfig {
    #[cfg(feature = "vdir")]
    Vdir(crate::config::VdirConfig),
    #[cfg(feature = "pimdir")]
    Pimdir(crate::config::PimdirConfig),
    #[cfg(feature = "carddav")]
    Carddav(Box<crate::config::CarddavConfig>),
    #[cfg(feature = "jmap")]
    Jmap(Box<crate::config::JmapConfig>),
    #[cfg(feature = "msgraph")]
    Msgraph(Box<crate::config::MsgraphConfig>),
    #[cfg(feature = "gpeople")]
    Gpeople(Box<crate::config::GpeopleConfig>),
    /// Stands in when no backend is compiled in: never built, so every
    /// match over the backends stays exhaustive.
    #[cfg(not(any(
        feature = "vdir",
        feature = "pimdir",
        feature = "carddav",
        feature = "jmap",
        feature = "msgraph",
        feature = "gpeople"
    )))]
    Unused(Infallible),
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
    #[cfg(feature = "gpeople")]
    Gpeople(Box<crate::gpeople::backend::GpeopleBackend>),
    /// Stands in when no backend is compiled in: never built, so every
    /// match over the backends stays exhaustive.
    #[cfg(not(any(
        feature = "vdir",
        feature = "pimdir",
        feature = "carddav",
        feature = "jmap",
        feature = "msgraph",
        feature = "gpeople"
    )))]
    Unused(Infallible),
}

impl AddressbookClient {
    /// Selects the backend from the account configuration.
    ///
    /// The first configured backend that `backend` allows wins, and the
    /// call bails when the account configures none of them, or fails
    /// coded `backend-not-built` when that first one is left out of the
    /// build ([`resolve_backend`]). Nothing is connected here: the first
    /// call that needs the network opens it.
    pub fn new(
        config: Config,
        name: String,
        #[allow(unused_mut)] mut account_config: AccountConfig,
        backend: Backend,
    ) -> Result<Self> {
        let inner = match resolve_backend(&account_config, backend)? {
            #[cfg(feature = "vdir")]
            Some(Backend::Vdir) => account_config.vdir.take().map(BackendConfig::Vdir),
            #[cfg(feature = "pimdir")]
            Some(Backend::Pimdir) => account_config.pimdir.take().map(BackendConfig::Pimdir),
            #[cfg(feature = "carddav")]
            Some(Backend::Carddav) => account_config
                .carddav
                .take()
                .map(|config| BackendConfig::Carddav(Box::new(config))),
            #[cfg(feature = "jmap")]
            Some(Backend::Jmap) => account_config
                .jmap
                .take()
                .map(|config| BackendConfig::Jmap(Box::new(config))),
            #[cfg(feature = "msgraph")]
            Some(Backend::Msgraph) => account_config
                .msgraph
                .take()
                .map(|config| BackendConfig::Msgraph(Box::new(config))),
            #[cfg(feature = "gpeople")]
            Some(Backend::Gpeople) => account_config
                .gpeople
                .take()
                .map(|config| BackendConfig::Gpeople(Box::new(config))),
            _ => None,
        };

        let Some(config_) = inner else {
            bail!("No backend matching `{backend}` is configured for this account");
        };

        let account = Account::from(config).merge(Account::from(account_config));

        Ok(Self {
            config: config_,
            inner: None,
            resolver: SecretResolver::new(),
            name,
            account,
        })
    }

    /// The backend the client speaks to, as `--backend` names it.
    pub fn backend(&self) -> Backend {
        match &self.config {
            #[cfg(not(any(
                feature = "vdir",
                feature = "pimdir",
                feature = "carddav",
                feature = "jmap",
                feature = "msgraph",
                feature = "gpeople"
            )))]
            BackendConfig::Unused(never) => match *never {},
            #[cfg(feature = "vdir")]
            BackendConfig::Vdir(_) => Backend::Vdir,
            #[cfg(feature = "pimdir")]
            BackendConfig::Pimdir(_) => Backend::Pimdir,
            #[cfg(feature = "carddav")]
            BackendConfig::Carddav(_) => Backend::Carddav,
            #[cfg(feature = "jmap")]
            BackendConfig::Jmap(_) => Backend::Jmap,
            #[cfg(feature = "msgraph")]
            BackendConfig::Msgraph(_) => Backend::Msgraph,
            #[cfg(feature = "gpeople")]
            BackendConfig::Gpeople(_) => Backend::Gpeople,
        }
    }

    /// Fails with `capability-missing` when the backend does not serve one
    /// of `capabilities`, before anything connects.
    pub fn require(&self, capabilities: &[Capability]) -> Result<()> {
        capability::require(&self.backend().to_string(), capabilities)
    }

    /// Whether the backend serves `capability` at any level but `none`.
    pub fn serves(&self, capability: Capability) -> bool {
        let support = capability::support(&self.backend().to_string(), capability);
        !matches!(support, capability::Support::None(_))
    }

    /// Whether the backend queues a write for a sync to push rather than
    /// make it on the server, so a write is confirmed as queued. Only the
    /// printed wording reads it, never the JSON.
    pub fn queues_writes(&self) -> bool {
        match &self.config {
            #[cfg(not(any(
                feature = "vdir",
                feature = "pimdir",
                feature = "carddav",
                feature = "jmap",
                feature = "msgraph",
                feature = "gpeople"
            )))]
            BackendConfig::Unused(never) => match *never {},
            #[cfg(feature = "vdir")]
            BackendConfig::Vdir(_) => false,
            #[cfg(feature = "pimdir")]
            BackendConfig::Pimdir(_) => true,
            #[cfg(feature = "carddav")]
            BackendConfig::Carddav(_) => false,
            #[cfg(feature = "jmap")]
            BackendConfig::Jmap(_) => false,
            #[cfg(feature = "msgraph")]
            BackendConfig::Msgraph(_) => false,
            #[cfg(feature = "gpeople")]
            BackendConfig::Gpeople(_) => false,
        }
    }

    /// `err` as the backend's configuration explains it: a CardDAV 401
    /// to a block with no `auth` says the block configures none.
    pub fn explain(&self, err: Error) -> Error {
        match &self.config {
            #[cfg(feature = "carddav")]
            BackendConfig::Carddav(config) => {
                crate::carddav::client::unauthenticated(err, &config.auth)
            }
            #[allow(unreachable_patterns)]
            _ => err,
        }
    }

    /// Drops the connection, so the next call opens a fresh one.
    ///
    /// Called before a composer runs: an editor session lasts minutes,
    /// and a server that closed the idle connection meanwhile would fail
    /// the write that comes after it.
    pub fn disconnect(&mut self) {
        if self.inner.take().is_some() {
            debug!("closing the backend connection");
        }
    }

    /// The open backend, opening it when it is not.
    fn open(&mut self) -> Result<&mut BackendClient> {
        if self.inner.is_none() {
            debug!("opening the backend connection");
            self.inner = Some(self.config.open(&mut self.resolver)?);
        }

        Ok(self.inner.as_mut().expect("just opened"))
    }

    /// Refuses, before anything connects, an address book id the backend
    /// cannot address and, when the call names one, a card id: on CardDAV
    /// an id io-webdav refuses, coded as an address book or a card the
    /// server does not hold.
    #[cfg_attr(not(feature = "carddav"), allow(unused_variables))]
    fn check_ids(&self, addressbook_id: &str, card_id: Option<&str>) -> Result<()> {
        match &self.config {
            #[cfg(feature = "carddav")]
            BackendConfig::Carddav(_) => {
                crate::carddav::client::check_addressbook_id(addressbook_id)?;
                card_id.map_or(Ok(()), |id| {
                    crate::carddav::client::check_card_id(addressbook_id, id)
                })
            }
            #[allow(unreachable_patterns)]
            _ => Ok(()),
        }
    }

    /// Refuses, before anything connects, the name of an address book to
    /// create the backend cannot address: on CardDAV, where the name is
    /// the collection's id, one io-webdav refuses.
    #[cfg_attr(not(feature = "carddav"), allow(unused_variables))]
    fn check_name(&self, name: &str) -> Result<()> {
        match &self.config {
            #[cfg(feature = "carddav")]
            BackendConfig::Carddav(_) => crate::carddav::client::check_addressbook_name(name),
            #[allow(unreachable_patterns)]
            _ => Ok(()),
        }
    }

    /// Opens the client on `backend`, a Graph client over a scripted
    /// stream, rather than connecting.
    #[cfg(all(test, feature = "msgraph"))]
    pub fn open_msgraph(&mut self, backend: crate::msgraph::backend::MsgraphBackend) {
        self.inner = Some(BackendClient::Msgraph(Box::new(backend)));
    }

    /// Opens the client on `backend`, a CardDAV client over a scripted
    /// stream, rather than connecting.
    #[cfg(all(test, feature = "carddav"))]
    pub fn open_carddav(&mut self, backend: crate::carddav::backend::CarddavBackend) {
        self.inner = Some(BackendClient::Carddav(Box::new(backend)));
    }
}

impl BackendConfig {
    /// Opens the backend this configuration describes, its credentials
    /// resolved through `resolver`.
    #[cfg_attr(
        not(any(
            feature = "carddav",
            feature = "jmap",
            feature = "msgraph",
            feature = "gpeople"
        )),
        allow(unused_variables)
    )]
    fn open(&self, resolver: &mut SecretResolver) -> Result<BackendClient> {
        match self {
            #[cfg(not(any(
                feature = "vdir",
                feature = "pimdir",
                feature = "carddav",
                feature = "jmap",
                feature = "msgraph",
                feature = "gpeople"
            )))]
            Self::Unused(never) => match *never {},
            #[cfg(feature = "vdir")]
            Self::Vdir(config) => {
                use crate::vdir::backend::VdirBackend;
                Ok(BackendClient::Vdir(VdirBackend::new(config.clone())))
            }
            #[cfg(feature = "pimdir")]
            Self::Pimdir(config) => {
                use crate::pimdir::backend::PimdirBackend;
                let client = PimdirBackend::new(config.clone())?;
                Ok(BackendClient::Pimdir(Box::new(client)))
            }
            #[cfg(feature = "carddav")]
            Self::Carddav(config) => {
                use crate::carddav::backend::CarddavBackend;
                let client = CarddavBackend::new(config.as_ref().clone(), resolver)?;
                Ok(BackendClient::Carddav(Box::new(client)))
            }
            #[cfg(feature = "jmap")]
            Self::Jmap(config) => {
                use crate::jmap::backend::JmapBackend;
                let client = JmapBackend::new(config.as_ref().clone(), resolver)?;
                Ok(BackendClient::Jmap(Box::new(client)))
            }
            #[cfg(feature = "msgraph")]
            Self::Msgraph(config) => {
                use crate::msgraph::backend::MsgraphBackend;
                let client = MsgraphBackend::new(config.as_ref().clone(), resolver)?;
                Ok(BackendClient::Msgraph(Box::new(client)))
            }
            #[cfg(feature = "gpeople")]
            Self::Gpeople(config) => {
                use crate::gpeople::backend::GpeopleBackend;
                let client = GpeopleBackend::new(config.as_ref().clone(), resolver)?;
                Ok(BackendClient::Gpeople(Box::new(client)))
            }
        }
    }
}

impl AddressbookClient {
    /// Lists every addressbook available to the active account.
    pub fn list_addressbooks(&mut self) -> Result<Vec<Addressbook>> {
        match self.open()? {
            #[cfg(not(any(
                feature = "vdir",
                feature = "pimdir",
                feature = "carddav",
                feature = "jmap",
                feature = "msgraph",
                feature = "gpeople"
            )))]
            BackendClient::Unused(never) => match *never {},
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
            #[cfg(feature = "gpeople")]
            BackendClient::Gpeople(client) => client.list_addressbooks(),
        }
    }

    /// Creates an addressbook and returns the id the backend assigned.
    ///
    /// `id` reaches the backends letting a client choose one, vdir and
    /// CardDAV, the capability check refusing it on the others.
    #[cfg_attr(
        not(any(feature = "vdir", feature = "carddav")),
        allow(unused_variables)
    )]
    pub fn create_addressbook(
        &mut self,
        id: Option<&str>,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<String> {
        self.check_name(id.unwrap_or(name))?;

        match self.open()? {
            #[cfg(not(any(
                feature = "vdir",
                feature = "pimdir",
                feature = "carddav",
                feature = "jmap",
                feature = "msgraph",
                feature = "gpeople"
            )))]
            BackendClient::Unused(never) => match *never {},
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.create_addressbook(id, name, description, color),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.create_addressbook(name, description, color),
            #[cfg(feature = "carddav")]
            BackendClient::Carddav(client) => {
                client.create_addressbook(id, name, description, color)
            }
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => client.create_addressbook(name, description, color),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.create_addressbook(name, description, color),
            #[cfg(feature = "gpeople")]
            BackendClient::Gpeople(client) => client.create_addressbook(name, description, color),
        }
    }

    /// Applies a partial update to the addressbook identified by `id`.
    pub fn update_addressbook(&mut self, id: &str, patch: AddressbookDiff) -> Result<()> {
        self.check_ids(id, None)?;

        match self.open()? {
            #[cfg(not(any(
                feature = "vdir",
                feature = "pimdir",
                feature = "carddav",
                feature = "jmap",
                feature = "msgraph",
                feature = "gpeople"
            )))]
            BackendClient::Unused(never) => match *never {},
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
            #[cfg(feature = "gpeople")]
            BackendClient::Gpeople(client) => client.update_addressbook(id, patch),
        }
    }

    /// Deletes the addressbook identified by `id` and every card it
    /// exclusively contains.
    pub fn delete_addressbook(&mut self, id: &str) -> Result<()> {
        self.check_ids(id, None)?;

        match self.open()? {
            #[cfg(not(any(
                feature = "vdir",
                feature = "pimdir",
                feature = "carddav",
                feature = "jmap",
                feature = "msgraph",
                feature = "gpeople"
            )))]
            BackendClient::Unused(never) => match *never {},
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
            #[cfg(feature = "gpeople")]
            BackendClient::Gpeople(client) => client.delete_addressbook(id),
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
    ) -> Result<CardPage> {
        let (skip, take) = match page_size {
            None => (0, None),
            Some(0) => return Ok(CardPage::default()),
            Some(size) => {
                let page = page.unwrap_or(1).max(1);
                let skip = ((page - 1) as usize).saturating_mul(size as usize);
                // NOTE: one card past the page tells whether another follows.
                (skip, Some(size as usize + 1))
            }
        };

        let window = self.list_cards_window(addressbook_id, skip, take)?;
        Ok(CardPage::of(window, take))
    }

    /// Lists the `take` cards of `addressbook_id` after the first `skip`,
    /// every card after them when `take` is `None`.
    fn list_cards_window(
        &mut self,
        addressbook_id: &str,
        skip: usize,
        take: Option<usize>,
    ) -> Result<CardWindow> {
        self.check_ids(addressbook_id, None)?;

        match self.open()? {
            #[cfg(not(any(
                feature = "vdir",
                feature = "pimdir",
                feature = "carddav",
                feature = "jmap",
                feature = "msgraph",
                feature = "gpeople"
            )))]
            BackendClient::Unused(never) => match *never {},
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.list_cards(addressbook_id, skip, take),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.list_cards(addressbook_id, skip, take),
            #[cfg(feature = "carddav")]
            BackendClient::Carddav(client) => client.list_cards(addressbook_id, skip, take),
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => client.list_cards(addressbook_id, skip, take),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client
                .list_cards(addressbook_id, skip, take)
                .map(CardWindow::from),
            #[cfg(feature = "gpeople")]
            BackendClient::Gpeople(client) => client
                .list_cards(addressbook_id, skip, take)
                .map(CardWindow::from),
        }
    }

    /// One page of the cards `search` matches: the index answers on
    /// pimdir, and every other backend has every card of the scope
    /// matched by cardamum.
    #[cfg(any(
        feature = "vdir",
        feature = "pimdir",
        feature = "carddav",
        feature = "jmap",
        feature = "msgraph",
        feature = "gpeople"
    ))]
    pub fn search_cards(
        &mut self,
        search: &crate::shared::card::search::CardSearch,
    ) -> Result<crate::shared::card::search::CardSearchPage> {
        use crate::shared::card::search;

        #[cfg(feature = "pimdir")]
        #[allow(irrefutable_let_patterns)]
        if let BackendClient::Pimdir(client) = self.open()? {
            return search::pimdir_page(client, search);
        }

        search::matched_page(self, search)
    }

    /// Fetches the card `card_id` from `addressbook_id`.
    pub fn get_card(&mut self, addressbook_id: &str, card_id: &str) -> Result<Card> {
        self.check_ids(addressbook_id, Some(card_id))?;

        match self.open()? {
            #[cfg(not(any(
                feature = "vdir",
                feature = "pimdir",
                feature = "carddav",
                feature = "jmap",
                feature = "msgraph",
                feature = "gpeople"
            )))]
            BackendClient::Unused(never) => match *never {},
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
            #[cfg(feature = "gpeople")]
            BackendClient::Gpeople(client) => client.get_card(addressbook_id, card_id),
        }
    }

    /// Appends a raw vCard and returns the id the backend assigned.
    pub fn create_card(&mut self, addressbook_id: &str, contents: Vec<u8>) -> Result<String> {
        self.check_ids(addressbook_id, None)?;

        match self.open()? {
            #[cfg(not(any(
                feature = "vdir",
                feature = "pimdir",
                feature = "carddav",
                feature = "jmap",
                feature = "msgraph",
                feature = "gpeople"
            )))]
            BackendClient::Unused(never) => match *never {},
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
            #[cfg(feature = "gpeople")]
            BackendClient::Gpeople(client) => client.create_card(addressbook_id, contents),
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
        self.check_ids(addressbook_id, Some(card_id))?;

        match self.open()? {
            #[cfg(not(any(
                feature = "vdir",
                feature = "pimdir",
                feature = "carddav",
                feature = "jmap",
                feature = "msgraph",
                feature = "gpeople"
            )))]
            BackendClient::Unused(never) => match *never {},
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
            #[cfg(feature = "gpeople")]
            BackendClient::Gpeople(client) => {
                client.update_card(addressbook_id, card_id, contents, if_match)
            }
        }
    }

    /// Permanently deletes `card_id` from `addressbook_id`.
    ///
    /// `if_match` gates the delete as it gates [`update_card`](Self::update_card);
    /// a backend with no such guard refuses it rather than pretending the
    /// check happened.
    pub fn delete_card(
        &mut self,
        addressbook_id: &str,
        card_id: &str,
        if_match: Option<&str>,
    ) -> Result<()> {
        self.check_ids(addressbook_id, Some(card_id))?;

        match self.open()? {
            #[cfg(not(any(
                feature = "vdir",
                feature = "pimdir",
                feature = "carddav",
                feature = "jmap",
                feature = "msgraph",
                feature = "gpeople"
            )))]
            BackendClient::Unused(never) => match *never {},
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.delete_card(addressbook_id, card_id, if_match),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.delete_card(addressbook_id, card_id, if_match),
            #[cfg(feature = "carddav")]
            BackendClient::Carddav(client) => client.delete_card(addressbook_id, card_id, if_match),
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => client.delete_card(addressbook_id, card_id, if_match),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.delete_card(addressbook_id, card_id, if_match),
            #[cfg(feature = "gpeople")]
            BackendClient::Gpeople(client) => client.delete_card(addressbook_id, card_id, if_match),
        }
    }
}

/// The `take` items of an in-memory list after the first `skip`, every
/// item after them when `take` is `None`.
#[cfg(any(
    feature = "carddav",
    feature = "msgraph",
    feature = "gpeople",
    feature = "vdir",
    all(test, feature = "pimdir")
))]
pub fn window<T>(items: Vec<T>, skip: usize, take: Option<usize>) -> Vec<T> {
    let items = items.into_iter().skip(skip);

    match take {
        Some(take) => items.take(take).collect(),
        None => items.collect(),
    }
}

/// The errors of a listing that reads the whole address book before
/// cutting its page: on the first page only, so no page repeats them.
#[cfg(any(feature = "carddav", feature = "vdir"))]
pub fn first_page(errors: Vec<ItemError>, skip: usize) -> Vec<ItemError> {
    match skip {
        0 => errors,
        _ => Vec::new(),
    }
}

/// Why `contents` reads as no vCard: a body that is not UTF-8, that does
/// not parse, or that opens no `BEGIN:VCARD` (the parser reads any
/// `name: value` text as lines); `None` for one that does.
pub fn unreadable(contents: &[u8]) -> Option<Error> {
    if let Err(err) = std::str::from_utf8(contents) {
        return Some(Error::new(err).context("Cannot read the card body as UTF-8"));
    }

    match VcardCst::parse(contents) {
        Ok(cst) if cst.begin.is_some() => None,
        Ok(_) => Some(anyhow!("No BEGIN:VCARD").context("Cannot parse the card body as a vCard")),
        Err(err) => Some(Error::new(err).context("Cannot parse the card body as a vCard")),
    }
}

/// The cards a backend listed for a page, beside the errors of the cards
/// it could not read.
#[derive(Debug, Default)]
pub struct CardWindow {
    /// The cards listed, in the backend's order. A card the backend
    /// names in `errors` may still hold its place here, so a page counts
    /// it; the page leaves it out. A card with no id and no contents is a
    /// place alone, held for a card the backend could not name, and the
    /// page leaves it out with no entry.
    pub cards: Vec<Card>,
    /// The error of each card the backend could not read, in the order
    /// met.
    pub errors: Vec<ItemError>,
}

impl CardPage {
    /// The page `window` holds, `take` being the page size plus one: the
    /// card past the page, when listed, tells that another follows and
    /// leaves with its entry. Each card left whose body reads as no vCard
    /// gains an entry, and a card with an entry is in `errors` alone.
    fn of(window: CardWindow, take: Option<usize>) -> Self {
        let CardWindow {
            mut cards,
            mut errors,
        } = window;
        let more = take.is_some_and(|take| cards.len() >= take);

        if more && let Some(next) = cards.pop() {
            errors.retain(|error| error.id.as_deref() != Some(next.id.as_str()));
        }

        // NOTE: the page is cut before a card leaves, so a card in `errors`
        // keeps its place and the next page starts where this one ended.
        cards.retain(|card| {
            if card.id.is_empty() && card.contents.is_empty() {
                return false;
            }

            let id = Some(card.id.as_str());
            if errors.iter().any(|error| error.id.as_deref() == id) {
                return false;
            }

            match unreadable(&card.contents) {
                Some(err) => {
                    errors.push(error::unreadable_card(id, err));
                    false
                }
                None => true,
            }
        });

        Self {
            cards,
            more,
            errors,
        }
    }
}

impl From<Vec<Card>> for CardWindow {
    fn from(cards: Vec<Card>) -> Self {
        Self {
            cards,
            errors: Vec::new(),
        }
    }
}

/// One page of a card listing.
#[derive(Debug, Default)]
pub struct CardPage {
    /// The cards of the page.
    pub cards: Vec<Card>,
    /// Whether more cards follow the page.
    pub more: bool,
    /// The error of each card the page could not read, in the order met.
    pub errors: Vec<ItemError>,
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;

    use super::*;
    use crate::error::ErrorCode;

    #[test]
    fn a_body_opening_no_vcard_is_unreadable() {
        assert!(unreadable(b"hello: world\n").is_some());
        assert!(unreadable(b"BEGIN:VCARD\r\nVERSION:4.0\r\nFN:A\r\nEND:VCARD\r\n").is_none());
    }

    fn card(id: &str, contents: &str) -> Card {
        Card {
            id: id.to_string(),
            addressbook_id: String::from("book"),
            etag: None,
            contents: contents.as_bytes().to_vec(),
        }
    }

    const CARD: &str = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Ada\r\nEND:VCARD\r\n";

    /// The card past a page leaves with its entry, which the next page
    /// prints. A card of the page whose body is no vCard gains one, and a
    /// card with an entry is in `errors` alone, whether its body parses
    /// (a pimdir preview) or not.
    #[test]
    fn a_card_the_page_could_not_read_is_in_its_errors_alone() {
        let entry = |id| error::unreadable_card(Some(id), anyhow!("Cannot read the card body"));
        let window = CardWindow {
            cards: vec![
                card("1", CARD),
                card("2", "not a card"),
                card("3", CARD),
                card("4", CARD),
            ],
            errors: vec![entry("3"), entry("4")],
        };

        let page = CardPage::of(window, Some(4));
        assert!(page.more);
        let ids: Vec<&str> = page.cards.iter().map(|card| card.id.as_str()).collect();
        assert_eq!(ids, ["1"]);
        let ids: Vec<_> = page.errors.iter().map(|error| error.id.clone()).collect();
        assert_eq!(ids, [Some("3".into()), Some("2".into())]);
        assert!(
            page.errors
                .iter()
                .all(|error| error.code == Some(ErrorCode::CardUnreadable))
        );
    }

    /// A card with no id and no contents holds a place the page counts,
    /// and leaves with no entry of its own.
    #[test]
    fn a_place_alone_is_counted_and_left_out() {
        let window = CardWindow {
            cards: vec![card("1", CARD), card("", ""), card("3", CARD)],
            errors: Vec::new(),
        };

        let page = CardPage::of(window, Some(3));
        assert!(page.more);
        let ids: Vec<&str> = page.cards.iter().map(|card| card.id.as_str()).collect();
        assert_eq!(ids, ["1"]);
        assert!(page.errors.is_empty());
    }
}
