//! JMAP arm of the shared-API client: thin glue mapping the shared
//! addressbook and card operations onto
//! [`io_jmap::client::JmapClientStd`] calls (RFC 9610), converting
//! ContactCards to vCard documents through calcard's JSContact codec
//! (see [`crate::jmap::project`]).
//!
//! JMAP AddressBooks are the addressbooks; ContactCards are
//! account-level with m:n memberships, so each listing queries the
//! requested AddressBook. Card updates carry no If-Match equivalent
//! server-side (last-write-wins), so passing one bails.

use std::collections::BTreeMap;

use anyhow::{Error, Result, anyhow, bail};
use base64::{Engine, prelude::BASE64_STANDARD};
use io_jmap::{
    client::JmapClientStd,
    rfc9610::{
        address_book::{
            JmapAddressBook, JmapAddressBookCreate, JmapAddressBookUpdate,
            get::JmapAddressBookGetOptions, set::JmapAddressBookSetArgs,
        },
        contact_card::{
            JmapContactCard, JmapContactCardFilter, JmapContactCardPatch,
            get::JmapContactCardGetOptions, query::JmapContactCardQueryOptions,
            set::JmapContactCardSetArgs,
        },
    },
};
use secrecy::{ExposeSecret, SecretString};

use crate::{
    config::{JmapAuthConfig, JmapConfig, parse_server},
    jmap::project,
    shared::{
        addressbook::{Addressbook, AddressbookDiff},
        card::Card,
        client::paginate,
    },
};

/// JMAP backend of the shared-API client, wrapping a connected
/// io-jmap client with its session discovered.
pub struct JmapBackend {
    pub inner: JmapClientStd,
}

impl JmapBackend {
    /// Establishes the JMAP session (TLS, `/.well-known/jmap`
    /// discovery) from the account's `[jmap]` block.
    pub fn new(config: JmapConfig) -> Result<Self> {
        let tls = config.tls.into_tls(config.alpn);
        let http_auth = jmap_http_auth(config.auth)?;
        let url = parse_server(&config.server, "https", &["http", "https", "jmap", "jmaps"])?;

        let mut inner = JmapClientStd::connect(&url, &tls, http_auth)?;
        inner.session_get(&url)?;

        Ok(Self { inner })
    }

    /// Lists the account's AddressBooks (RFC 9610 §2.1).
    pub fn list_addressbooks(&mut self) -> Result<Vec<Addressbook>> {
        let out = self
            .inner
            .address_book_get(JmapAddressBookGetOptions::default())?;

        Ok(out
            .address_books
            .into_iter()
            .map(into_addressbook)
            .collect())
    }

    /// Creates an AddressBook named `name`. JMAP AddressBooks carry no
    /// color, so passing one bails. Returns the server-assigned id.
    pub fn create_addressbook(
        &mut self,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<String> {
        if color.is_some() {
            bail!("JMAP AddressBooks do not support color");
        }

        let create = BTreeMap::from([(
            "c0".to_string(),
            JmapAddressBookCreate {
                name: Some(name.to_string()),
                description: description.map(str::to_string),
                ..Default::default()
            },
        )]);
        let args = JmapAddressBookSetArgs {
            create: Some(create),
            ..Default::default()
        };
        let out = self.inner.address_book_set(args)?;

        if let Some(err) = out.not_created.into_values().next() {
            bail!("JMAP AddressBook create rejected: {err:?}");
        }

        out.created
            .into_values()
            .next()
            .and_then(|created| created.id)
            .ok_or_else(|| anyhow!("JMAP create response is missing the AddressBook id"))
    }

    /// Applies `patch` to the AddressBook identified by `id`. JMAP
    /// AddressBooks carry no color, so patching one bails.
    pub fn update_addressbook(&mut self, id: &str, patch: AddressbookDiff) -> Result<()> {
        if patch.color.is_some() {
            bail!("JMAP AddressBooks do not support color");
        }

        let update = BTreeMap::from([(
            id.to_string(),
            JmapAddressBookUpdate {
                name: patch.name,
                description: patch.description.flatten(),
                ..Default::default()
            },
        )]);
        let args = JmapAddressBookSetArgs {
            update: Some(update),
            ..Default::default()
        };
        let out = self.inner.address_book_set(args)?;

        if let Some(err) = out.not_updated.into_values().next() {
            bail!("JMAP AddressBook update rejected: {err:?}");
        }

        Ok(())
    }

    /// Destroys the AddressBook identified by `id` along with the
    /// ContactCards it exclusively contains (RFC 9610 §2.3).
    pub fn delete_addressbook(&mut self, id: &str) -> Result<()> {
        let args = JmapAddressBookSetArgs {
            destroy: Some(vec![id.to_string()]),
            on_destroy_remove_contents: Some(true),
            ..Default::default()
        };
        let out = self.inner.address_book_set(args)?;

        if let Some(err) = out.not_destroyed.into_values().next() {
            bail!("JMAP AddressBook destroy rejected: {err:?}");
        }

        Ok(())
    }

    /// Lists the ContactCards of the AddressBook, each converted to a
    /// vCard document, applying 1-indexed pagination.
    pub fn list_cards(
        &mut self,
        addressbook_id: &str,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<Vec<Card>> {
        let opts = JmapContactCardQueryOptions {
            filter: Some(JmapContactCardFilter {
                in_address_book: Some(addressbook_id.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let out = self.inner.contact_card_query(opts)?;

        let cards = out
            .cards
            .into_iter()
            .map(|card| project::to_card(addressbook_id, card))
            .collect::<Result<Vec<_>, _>>()
            .map_err(Error::msg)?;

        Ok(paginate(cards, page, page_size))
    }

    /// Reads the ContactCard `card_id`, converted to a vCard document.
    pub fn get_card(&mut self, addressbook_id: &str, card_id: &str) -> Result<Card> {
        let opts = JmapContactCardGetOptions {
            ids: Some(vec![card_id.to_string()]),
            ..Default::default()
        };
        let out = self.inner.contact_card_get(opts)?;

        let card = out
            .cards
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("JMAP ContactCard `{card_id}` not found"))?;

        project::to_card(addressbook_id, card).map_err(Error::msg)
    }

    /// Creates the vCard as a ContactCard in the AddressBook. The
    /// server names the card, so the returned id is server-assigned.
    pub fn create_card(&mut self, addressbook_id: &str, contents: Vec<u8>) -> Result<String> {
        let vcard = into_vcard_text(contents)?;
        let card = project::to_jscontact(&vcard).map_err(Error::msg)?;

        let create = BTreeMap::from([(
            "c0".to_string(),
            JmapContactCard {
                id: None,
                address_book_ids: BTreeMap::from([(addressbook_id.to_string(), true)]),
                card,
            },
        )]);
        let args = JmapContactCardSetArgs {
            create: Some(create),
            ..Default::default()
        };
        let out = self.inner.contact_card_set(args)?;

        if let Some(err) = out.not_created.into_values().next() {
            bail!("JMAP ContactCard create rejected: {err:?}");
        }

        out.created
            .into_values()
            .next()
            .and_then(|created| created.id)
            .ok_or_else(|| anyhow!("JMAP create response is missing the card id"))
    }

    /// Updates the ContactCard `card_id` from the vCard. The current
    /// server card serves as patch base, so the patch shrinks to the
    /// changed properties, plus nulls for the removed ones. JMAP has
    /// no If-Match guard (updates are last-write-wins), so passing one
    /// bails instead of pretending to honor it.
    pub fn update_card(
        &mut self,
        addressbook_id: &str,
        card_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
    ) -> Result<()> {
        if if_match.is_some() {
            bail!("JMAP does not support If-Match guarded updates");
        }

        let vcard = into_vcard_text(contents)?;
        let base = self.get_card(addressbook_id, card_id)?;
        let base_vcard = into_vcard_text(base.contents)?;

        let patch = project::to_patch(&vcard, Some(&base_vcard)).map_err(Error::msg)?;

        let update = BTreeMap::from([(card_id.to_string(), JmapContactCardPatch(patch))]);
        let args = JmapContactCardSetArgs {
            update: Some(update),
            ..Default::default()
        };
        let out = self.inner.contact_card_set(args)?;

        if let Some(err) = out.not_updated.into_values().next() {
            bail!("JMAP ContactCard update rejected: {err:?}");
        }

        Ok(())
    }

    /// Destroys the ContactCard `card_id`.
    pub fn delete_card(&mut self, _addressbook_id: &str, card_id: &str) -> Result<()> {
        let args = JmapContactCardSetArgs {
            destroy: Some(vec![card_id.to_string()]),
            ..Default::default()
        };
        let out = self.inner.contact_card_set(args)?;

        if let Some(err) = out.not_destroyed.into_values().next() {
            bail!("JMAP ContactCard destroy rejected: {err:?}");
        }

        Ok(())
    }
}

/// Converts a [`JmapAuthConfig`] into the pre-formatted HTTP
/// `Authorization` header value [`JmapClientStd::connect`] expects.
pub fn jmap_http_auth(config: JmapAuthConfig) -> Result<SecretString> {
    match config {
        JmapAuthConfig::Header(header) => Ok(header.get()?),
        JmapAuthConfig::Bearer { token } => {
            let token = token.get()?;
            Ok(format!("Bearer {}", token.expose_secret()).into())
        }
        JmapAuthConfig::Basic { username, password } => {
            let creds = format!("{}:{}", username, password.get()?.expose_secret());
            let encoded = BASE64_STANDARD.encode(creds.into_bytes());
            Ok(format!("Basic {encoded}").into())
        }
    }
}

/// Maps a JMAP [`JmapAddressBook`] to the shared shape: the name falls
/// back to the id.
fn into_addressbook(book: JmapAddressBook) -> Addressbook {
    let id = book.id.unwrap_or_default();
    let name = book.name.filter(|name| !name.is_empty());

    Addressbook {
        name: name.unwrap_or_else(|| id.clone()),
        id,
        description: book.description,
        color: None,
    }
}

/// Decodes raw card bytes as UTF-8 vCard text.
fn into_vcard_text(contents: Vec<u8>) -> Result<String> {
    String::from_utf8(contents).map_err(|_| anyhow::anyhow!("Card contents are not valid UTF-8"))
}
