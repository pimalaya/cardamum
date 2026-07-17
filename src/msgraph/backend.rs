//! Microsoft Graph arm of the shared-API client: thin glue mapping the
//! shared addressbook and card operations onto
//! [`io_msgraph::v1::client::MsgraphClientStd`] calls, projecting Graph
//! contacts onto vCard documents (see [`crate::msgraph::project`]).
//!
//! Graph contact folders are the addressbooks; the default Contacts
//! folder is not listed by the folders endpoint, so it is surfaced
//! under the [`CONTACTS_FOLDER`] sentinel id. Card updates carry no
//! If-Match guard server-side (last-write-wins), so passing one bails.

use anyhow::{Error, Result, bail};
use io_msgraph::v1::{
    client::{MsgraphClientStd, MsgraphClientStdConnectOptions},
    rest::users::{
        contact_folders::MsgraphContactFolder,
        contacts::{MsgraphContact, list::MsgraphContactsListParams},
    },
    send::MsgraphSend,
};
use secrecy::ExposeSecret;
use url::Url;

use crate::{
    config::MsgraphConfig,
    msgraph::project,
    shared::{
        addressbook::{Addressbook, AddressbookDiff},
        card::Card,
        client::paginate,
    },
};

/// Sentinel addressbook id of the default Contacts folder, which Graph
/// addresses by omitting the folder segment.
pub const CONTACTS_FOLDER: &str = "contacts";

/// Microsoft Graph backend of the shared-API client.
pub struct MsgraphBackend {
    pub inner: MsgraphClientStd,
}

impl MsgraphBackend {
    /// Connects to the Graph API from the account's `[msgraph]` block.
    pub fn new(config: MsgraphConfig) -> Result<Self> {
        let token = config.auth.token.get()?;
        let options = MsgraphClientStdConnectOptions {
            tls: config.tls.into_tls(config.alpn),
            user_id: config.user_id,
        };
        let inner = MsgraphClientStd::connect(token.expose_secret(), options)?;
        Ok(Self { inner })
    }

    /// Lists the contact folders as addressbooks, the default Contacts
    /// folder first under the [`CONTACTS_FOLDER`] sentinel id.
    pub fn list_addressbooks(&mut self) -> Result<Vec<Addressbook>> {
        let mut books = vec![Addressbook {
            id: CONTACTS_FOLDER.to_string(),
            name: "Contacts".to_string(),
            description: None,
            color: None,
        }];

        let mut page = self
            .inner
            .contact_folders_list(&Default::default())?
            .response;

        loop {
            for folder in page.value {
                books.push(Addressbook {
                    name: if folder.display_name.is_empty() {
                        folder.id.clone()
                    } else {
                        folder.display_name
                    },
                    id: folder.id,
                    description: None,
                    color: None,
                });
            }

            match page.next_link {
                Some(next) => {
                    let url = parse_graph_url(&next)?;
                    let auth = self.inner.auth.clone();
                    page = self.inner.run(MsgraphSend::get(&auth, url))?.response;
                }
                None => break,
            }
        }

        Ok(books)
    }

    /// Creates a contact folder named `name`. Graph folders carry no
    /// description nor color, so passing either bails.
    pub fn create_addressbook(
        &mut self,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<String> {
        if description.is_some() || color.is_some() {
            bail!("Microsoft Graph contact folders support neither description nor color");
        }

        let folder = MsgraphContactFolder {
            display_name: name.to_string(),
            ..Default::default()
        };
        let created = self.inner.contact_folder_create(&folder)?.response;

        Ok(created.id)
    }

    /// Renames the contact folder identified by `id`. Graph folders
    /// carry no description nor color, so patching either bails.
    pub fn update_addressbook(&mut self, id: &str, patch: AddressbookDiff) -> Result<()> {
        if patch.description.is_some() || patch.color.is_some() {
            bail!("Microsoft Graph contact folders support neither description nor color");
        }
        if id == CONTACTS_FOLDER {
            bail!("The default Contacts folder cannot be updated");
        }

        let Some(name) = patch.name else {
            return Ok(());
        };

        let folder = MsgraphContactFolder {
            display_name: name,
            ..Default::default()
        };
        self.inner.contact_folder_update(id, &folder)?;

        Ok(())
    }

    /// Deletes the contact folder identified by `id` and every contact
    /// it contains.
    pub fn delete_addressbook(&mut self, id: &str) -> Result<()> {
        if id == CONTACTS_FOLDER {
            bail!("The default Contacts folder cannot be deleted");
        }

        self.inner.contact_folder_delete(id)?;
        Ok(())
    }

    /// Lists the contacts of the folder, each projected onto a vCard
    /// document, applying 1-indexed pagination.
    pub fn list_cards(
        &mut self,
        addressbook_id: &str,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<Vec<Card>> {
        let expand = graph_expand();
        let params = MsgraphContactsListParams {
            top: Some(100),
            expand: Some(&expand),
            ..Default::default()
        };

        let mut current = self
            .inner
            .contacts_list(folder(addressbook_id), &params)?
            .response;

        let mut cards = Vec::new();
        loop {
            cards.extend(
                current
                    .value
                    .into_iter()
                    .map(|contact| into_card(addressbook_id, contact)),
            );

            match current.next_link {
                Some(next) => {
                    let url = parse_graph_url(&next)?;
                    let auth = self.inner.auth.clone();
                    current = self.inner.run(MsgraphSend::get(&auth, url))?.response;
                }
                None => break,
            }
        }

        Ok(paginate(cards, page, page_size))
    }

    /// Reads the contact `card_id`, projected onto a vCard document.
    pub fn get_card(&mut self, addressbook_id: &str, card_id: &str) -> Result<Card> {
        let expand = graph_expand();
        let contact = self.inner.contact_get(card_id, Some(&expand))?.response;
        Ok(into_card(addressbook_id, contact))
    }

    /// Creates the vCard as a Graph contact in the folder. Graph names
    /// the resource, so the returned id is server-assigned.
    pub fn create_card(&mut self, addressbook_id: &str, contents: Vec<u8>) -> Result<String> {
        let vcard = into_vcard_text(contents)?;
        let contact = project::to_new_contact(&vcard).map_err(Error::msg)?;

        let created = self
            .inner
            .contact_create(folder(addressbook_id), &contact)?
            .response;

        Ok(created.id)
    }

    /// Updates the contact `card_id` from the vCard. The current server
    /// contact serves as delta base, so the PATCH body shrinks to the
    /// changed fields, plus nulls for the removed ones. Graph has no
    /// If-Match guard (updates are last-write-wins), so passing one
    /// bails instead of pretending to honor it.
    pub fn update_card(
        &mut self,
        _addressbook_id: &str,
        card_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
    ) -> Result<()> {
        if if_match.is_some() {
            bail!("Microsoft Graph does not support If-Match guarded updates");
        }

        let vcard = into_vcard_text(contents)?;

        let expand = graph_expand();
        let base = self.inner.contact_get(card_id, Some(&expand))?.response;
        let base_vcard = project::to_vcard(&base);

        let contact = project::to_contact_delta(&vcard, &base_vcard).map_err(Error::msg)?;
        self.inner.contact_update(card_id, &contact)?;

        Ok(())
    }

    /// Deletes the contact `card_id`.
    pub fn delete_card(&mut self, _addressbook_id: &str, card_id: &str) -> Result<()> {
        self.inner.contact_delete(card_id)?;
        Ok(())
    }
}

/// Maps the sentinel Contacts folder id to the omitted folder segment.
fn folder(addressbook_id: &str) -> Option<&str> {
    (addressbook_id != CONTACTS_FOLDER).then_some(addressbook_id)
}

/// io-msgraph contact to the shared card shape: the projected vCard
/// document as contents, the Graph id as id and the changeKey as ETag.
fn into_card(addressbook_id: &str, contact: MsgraphContact) -> Card {
    let vcard = project::to_vcard(&contact);
    Card {
        id: contact.id,
        addressbook_id: addressbook_id.to_string(),
        etag: contact.change_key,
        contents: vcard.into_bytes(),
    }
}

/// Decodes raw card bytes as UTF-8 vCard text.
fn into_vcard_text(contents: Vec<u8>) -> Result<String> {
    String::from_utf8(contents).map_err(|_| anyhow::anyhow!("Card contents are not valid UTF-8"))
}

/// Parses an OData paging link served by Graph.
fn parse_graph_url(raw: &str) -> Result<Url> {
    Url::parse(raw).map_err(|err| anyhow::anyhow!("Invalid Graph page URL `{raw}`: {err}"))
}

/// The `$expand` clause fetching the stash extended property along with
/// the contact (Graph omits extended properties otherwise).
fn graph_expand() -> String {
    format!(
        "singleValueExtendedProperties($filter=id eq '{}')",
        project::EXTENDED_PROP_ID
    )
}
