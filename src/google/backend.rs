//! Google People arm of the shared-API client: thin glue mapping the
//! shared addressbook and card operations onto
//! [`io_people::v1::client::PeopleClientStd`] calls, projecting
//! People persons onto vCard documents (see [`crate::google::project`]).
//!
//! Contact groups are the addressbooks: the myContacts system group
//! (the group every contact belongs to) is surfaced first as Contacts,
//! then the user's own groups. Memberships are m:n labels, so one card
//! can appear under several books; the shared API narrows each listing
//! to the requested group.

use anyhow::{Error, Result, bail};
use io_people::v1::{
    client::{PeopleClientStd, PeopleClientStdConnectOptions},
    rest::{
        contact_groups::{
            PeopleContactGroup, PeopleContactGroupType, list::PeopleContactGroupsListParams,
        },
        people::{PeoplePerson, PeoplePersonField, connections::list::PeopleConnectionsListParams},
    },
};
use secrecy::ExposeSecret;

use crate::{
    config::GoogleConfig,
    google::project,
    shared::{
        addressbook::{Addressbook, AddressbookDiff},
        card::Card,
        client::paginate,
    },
};

/// Contact group id of the myContacts system group, the container
/// every Google contact belongs to.
pub const MY_CONTACTS_GROUP: &str = "myContacts";

/// Google People backend of the shared-API client.
pub struct GoogleBackend {
    pub inner: PeopleClientStd,
}

impl GoogleBackend {
    /// Connects to the People API from the account's `[google]` block.
    pub fn new(config: GoogleConfig) -> Result<Self> {
        let token = config.auth.token.get()?;
        let options = PeopleClientStdConnectOptions {
            tls: config.tls.into_tls(config.alpn),
        };
        let inner = PeopleClientStd::connect(token.expose_secret(), options)?;
        Ok(Self { inner })
    }

    /// Lists the account's contact groups as addressbooks: the
    /// myContacts system group first (as Contacts), then the user's own
    /// groups.
    pub fn list_addressbooks(&mut self) -> Result<Vec<Addressbook>> {
        let mut books = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let params = PeopleContactGroupsListParams {
                page_token: page_token.as_deref(),
                ..Default::default()
            };
            let page = self.inner.contact_groups_list(&[], &params)?.response;

            for group in page.contact_groups {
                if group.metadata.as_ref().and_then(|m| m.deleted) == Some(true) {
                    continue;
                }

                let id = group_id(&group.resource_name).to_string();
                if id.is_empty() {
                    continue;
                }

                // NOTE: of the system groups, only myContacts is a
                // container (starred, blocked and friends-style legacy
                // groups are not addressbooks).
                if id == MY_CONTACTS_GROUP {
                    books.insert(
                        0,
                        Addressbook {
                            id,
                            name: "Contacts".to_string(),
                            description: None,
                            color: None,
                        },
                    );
                } else if group.group_type == Some(PeopleContactGroupType::UserContactGroup) {
                    let name = group
                        .name
                        .or(group.formatted_name)
                        .unwrap_or_else(|| id.clone());
                    books.push(Addressbook {
                        id,
                        name,
                        description: None,
                        color: None,
                    });
                }
            }

            match page.next_page_token {
                Some(next) => page_token = Some(next),
                None => break,
            }
        }

        Ok(books)
    }

    /// Creates a user contact group named `name`. Groups carry no
    /// description nor color, so passing either bails.
    pub fn create_addressbook(
        &mut self,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<String> {
        if description.is_some() || color.is_some() {
            bail!("Google contact groups support neither description nor color");
        }

        let group = PeopleContactGroup {
            name: Some(name.to_string()),
            ..Default::default()
        };
        let created = self.inner.contact_group_create(&group, &[])?.response;

        Ok(group_id(&created.resource_name).to_string())
    }

    /// Renames the user contact group identified by `id`. Groups carry
    /// no description nor color, so patching either bails. The update
    /// is guarded by the group's current etag, fetched first.
    pub fn update_addressbook(&mut self, id: &str, patch: AddressbookDiff) -> Result<()> {
        if patch.description.is_some() || patch.color.is_some() {
            bail!("Google contact groups support neither description nor color");
        }
        if id == MY_CONTACTS_GROUP {
            bail!("The Contacts system group cannot be updated");
        }

        let Some(name) = patch.name else {
            return Ok(());
        };

        let resource_name = format!("contactGroups/{id}");
        let current = self
            .inner
            .contact_group_get(&resource_name, None, &[])?
            .response;

        let group = PeopleContactGroup {
            resource_name,
            etag: current.etag,
            name: Some(name),
            ..Default::default()
        };
        self.inner.contact_group_update(&group, &[], &[])?;

        Ok(())
    }

    /// Deletes the user contact group identified by `id`; its contacts
    /// stay in myContacts.
    pub fn delete_addressbook(&mut self, id: &str) -> Result<()> {
        if id == MY_CONTACTS_GROUP {
            bail!("The Contacts system group cannot be deleted");
        }

        self.inner
            .contact_group_delete(&format!("contactGroups/{id}"), false)?;
        Ok(())
    }

    /// Lists the contacts of the group, each projected onto a vCard
    /// document, applying 1-indexed pagination.
    pub fn list_cards(
        &mut self,
        addressbook_id: &str,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<Vec<Card>> {
        let mut cards = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let params = PeopleConnectionsListParams {
                page_size: Some(100),
                page_token: page_token.as_deref(),
                ..Default::default()
            };
            let current = self
                .inner
                .connections_list(project::READ_FIELDS, &params)?
                .response;

            cards.extend(
                current
                    .connections
                    .into_iter()
                    .filter(|person| in_group(person, addressbook_id))
                    .map(|person| into_card(addressbook_id, person)),
            );

            match current.next_page_token {
                Some(next) => page_token = Some(next),
                None => break,
            }
        }

        Ok(paginate(cards, page, page_size))
    }

    /// Reads the contact `card_id`, projected onto a vCard document.
    pub fn get_card(&mut self, addressbook_id: &str, card_id: &str) -> Result<Card> {
        let person = self
            .inner
            .person_get(&format!("people/{card_id}"), project::READ_FIELDS, &[])?
            .response;

        Ok(into_card(addressbook_id, person))
    }

    /// Creates the vCard as a People contact; creates always land in
    /// myContacts, so a user group target adds the membership right
    /// after. Returns the server-assigned id.
    pub fn create_card(&mut self, addressbook_id: &str, contents: Vec<u8>) -> Result<String> {
        let vcard = into_vcard_text(contents)?;
        let person = project::to_person(&vcard).map_err(Error::msg)?;

        let created = self
            .inner
            .contact_create(&person, project::READ_FIELDS, &[])?
            .response;
        let id = project::person_id(&created.resource_name).to_string();

        if addressbook_id != MY_CONTACTS_GROUP {
            let modified = self
                .inner
                .contact_group_members_modify(
                    &format!("contactGroups/{addressbook_id}"),
                    &[created.resource_name],
                    &[],
                )?
                .response;

            if !modified.not_found_resource_names.is_empty() {
                bail!(
                    "Google group member add rejected: {:?} not found",
                    modified.not_found_resource_names
                );
            }
        }

        Ok(id)
    }

    /// Updates the contact `card_id` from the vCard. The current server
    /// person serves as delta base, so the update mask shrinks to the
    /// changed fields. People requires the person's current etag on
    /// updates; `if_match` supplies it, otherwise the fetched one is
    /// used. A stash write (clientData in the mask) merges the server's
    /// foreign clientData entries under the same guard.
    pub fn update_card(
        &mut self,
        _addressbook_id: &str,
        card_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
    ) -> Result<()> {
        let vcard = into_vcard_text(contents)?;
        let resource_name = format!("people/{card_id}");

        let mut person = project::to_person(&vcard).map_err(Error::msg)?;
        person.resource_name = resource_name.clone();

        let current = self
            .inner
            .person_get(&resource_name, project::READ_FIELDS, &[])?
            .response;

        let base = project::to_vcard(&current);
        let base_person = project::to_person(&base).map_err(Error::msg)?;
        let fields = project::changed_fields(&person, &base_person);
        if fields.is_empty() {
            // NOTE: nothing differs from the server state, no request
            // to send.
            return Ok(());
        }

        // NOTE: a masked update replaces the whole clientData list and
        // other clients may own entries there, so a stash write merges
        // the server's foreign entries first (the etag guard turns a
        // lost-update race into a clean rejection).
        if fields.contains(&PeoplePersonField::ClientData) {
            let mut merged: Vec<_> = current
                .client_data
                .into_iter()
                .filter(|entry| entry.key.as_deref() != Some(project::CLIENT_DATA_KEY))
                .collect();
            merged.append(&mut person.client_data);
            person.client_data = merged;
        }

        person.etag = match if_match {
            Some(etag) => etag.to_string(),
            None => current.etag,
        };

        self.inner
            .contact_update(&person, &fields, project::READ_FIELDS, &[])?;

        Ok(())
    }

    /// Deletes the contact `card_id`.
    pub fn delete_card(&mut self, _addressbook_id: &str, card_id: &str) -> Result<()> {
        self.inner.contact_delete(&format!("people/{card_id}"))?;
        Ok(())
    }
}

/// Whether the person is a member of the contact group `id`.
fn in_group(person: &PeoplePerson, id: &str) -> bool {
    person
        .memberships
        .iter()
        .filter_map(|membership| membership.contact_group_membership.as_ref())
        .filter_map(|group| {
            group
                .contact_group_resource_name
                .as_deref()
                .map(group_id)
                .or(group.contact_group_id.as_deref())
        })
        .any(|group| group == id)
}

/// io-people person to the shared card shape: the projected
/// vCard document as contents, the person id as id and the person etag
/// as ETag.
fn into_card(addressbook_id: &str, person: PeoplePerson) -> Card {
    let vcard = project::to_vcard(&person);
    let etag = (!person.etag.is_empty()).then(|| person.etag.clone());

    Card {
        id: project::person_id(&person.resource_name).to_string(),
        addressbook_id: addressbook_id.to_string(),
        etag,
        contents: vcard.into_bytes(),
    }
}

/// Strips the `contactGroups/` prefix off a group resource name.
fn group_id(resource_name: &str) -> &str {
    resource_name
        .strip_prefix("contactGroups/")
        .unwrap_or(resource_name)
}

/// Decodes raw card bytes as UTF-8 vCard text.
fn into_vcard_text(contents: Vec<u8>) -> Result<String> {
    String::from_utf8(contents).map_err(|_| anyhow::anyhow!("Card contents are not valid UTF-8"))
}
