//! # Google People backend
//!
//! The People arm of the shared-API client: thin glue mapping the shared
//! addressbook and card operations onto
//! [`io_gpeople::v1::client::GpeopleClientStd`] calls, projecting persons
//! onto vCard documents through io-gpeople's `vcard` feature.
//!
//! Contact groups are the addressbooks: the myContacts system group, the
//! one every contact belongs to, comes first as Contacts, then the
//! user's own groups. Memberships are m:n labels, so one card can appear
//! under several books and each listing is narrowed to its group.

use anyhow::{Result, bail};
use io_gpeople::v1::{
    client::{GpeopleClientStd, GpeopleClientStdConnectOptions},
    rest::{
        contact_groups::{
            GpeopleContactGroup, GpeopleContactGroupType, list::GpeopleContactGroupsListParams,
        },
        people::{
            GpeoplePerson, GpeoplePersonField,
            connections::list::GpeopleConnectionsListParams,
            vcard::{GPEOPLE_PERSON_STASH_KEY, GPEOPLE_PERSON_VCARD_FIELDS},
        },
    },
};
use pimalaya_config::secret::SecretResolver;
use secrecy::ExposeSecret;

use crate::{
    config::{GpeopleConfig, ProxyConfig},
    shared::{
        addressbook::{Addressbook, AddressbookDiff},
        card::{Card, CardUpdateOutcome},
        client::paginate,
    },
};

/// Contact group id of myContacts, the group every contact belongs to.
pub const MY_CONTACTS_GROUP: &str = "myContacts";

/// Google People backend of the shared-API client.
pub struct GpeopleBackend {
    pub inner: GpeopleClientStd,
}

impl GpeopleBackend {
    /// Connects to the People API from the account's `[gpeople]` block.
    ///
    /// The credential is resolved through `resolver`, so an account naming
    /// one credential command from several of its backends spawns it once.
    pub fn new(config: GpeopleConfig, resolver: &mut SecretResolver) -> Result<Self> {
        let token = resolver.resolve(config.auth.token)?;
        let options = GpeopleClientStdConnectOptions {
            tls: config.tls.into_tls(config.alpn),
            proxy: ProxyConfig::resolve(config.proxy, resolver)?,
        };
        let inner = GpeopleClientStd::connect(token.expose_secret(), options)?;
        Ok(Self { inner })
    }

    /// Lists the account's contact groups as addressbooks.
    ///
    /// The myContacts system group comes first, as Contacts, then the
    /// user's own groups.
    pub fn list_addressbooks(&mut self) -> Result<Vec<Addressbook>> {
        let mut books = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let params = GpeopleContactGroupsListParams {
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
                // container: starred, blocked and the legacy ones are
                // not addressbooks.
                if id == MY_CONTACTS_GROUP {
                    books.insert(
                        0,
                        Addressbook {
                            id,
                            name: "Contacts".to_string(),
                            description: None,
                            color: None,
                            default: false,
                        },
                    );
                } else if group.group_type == Some(GpeopleContactGroupType::UserContactGroup) {
                    let name = group
                        .name
                        .or(group.formatted_name)
                        .unwrap_or_else(|| id.clone());
                    books.push(Addressbook {
                        id,
                        name,
                        description: None,
                        color: None,
                        default: false,
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

    /// Creates a user contact group named `name`.
    ///
    /// Groups carry no description nor color, so passing either bails.
    pub fn create_addressbook(
        &mut self,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<String> {
        if description.is_some() || color.is_some() {
            bail!("Google contact groups support neither description nor color");
        }

        let group = GpeopleContactGroup {
            name: Some(name.to_string()),
            ..Default::default()
        };
        let created = self.inner.contact_group_create(&group, &[])?.response;

        Ok(group_id(&created.resource_name).to_string())
    }

    /// Renames the user contact group identified by `id`.
    ///
    /// Groups carry no description nor color, so patching either bails.
    /// The update is guarded by the group's current etag, fetched first.
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

        let group = GpeopleContactGroup {
            resource_name,
            etag: current.etag,
            name: Some(name),
            ..Default::default()
        };
        self.inner.contact_group_update(&group, &[], &[])?;

        Ok(())
    }

    /// Deletes the user contact group identified by `id`.
    ///
    /// Its contacts stay in myContacts.
    pub fn delete_addressbook(&mut self, id: &str) -> Result<()> {
        if id == MY_CONTACTS_GROUP {
            bail!("The Contacts system group cannot be deleted");
        }

        self.inner
            .contact_group_delete(&format!("contactGroups/{id}"), false)?;
        Ok(())
    }

    /// Lists the contacts of the group, each projected onto a vCard.
    pub fn list_cards(
        &mut self,
        addressbook_id: &str,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<Vec<Card>> {
        let mut cards = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let params = GpeopleConnectionsListParams {
                page_size: Some(100),
                page_token: page_token.as_deref(),
                ..Default::default()
            };
            let current = self
                .inner
                .connections_list(GPEOPLE_PERSON_VCARD_FIELDS, &params)?
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
            .person_get(
                &format!("people/{card_id}"),
                GPEOPLE_PERSON_VCARD_FIELDS,
                &[],
            )?
            .response;

        Ok(into_card(addressbook_id, person))
    }

    /// Creates the vCard as a People contact, returning its new id.
    ///
    /// Creates always land in myContacts, so a user group target adds
    /// the membership right after.
    pub fn create_card(&mut self, addressbook_id: &str, contents: Vec<u8>) -> Result<String> {
        let vcard = into_vcard_text(contents)?;
        let person = GpeoplePerson::from_vcard(&vcard)?;

        let created = self
            .inner
            .contact_create(&person, GPEOPLE_PERSON_VCARD_FIELDS, &[])?
            .response;
        let id = created.id().to_string();

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

    /// Updates the contact `card_id` from the vCard.
    ///
    /// The mask shrinks to the fields differing from the server person,
    /// whose etag guards the write unless `if_match` gives one. The
    /// returned properties are stash entries Google refuses to drop.
    pub fn update_card(
        &mut self,
        _addressbook_id: &str,
        card_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
    ) -> Result<CardUpdateOutcome> {
        let vcard = into_vcard_text(contents)?;
        let resource_name = format!("people/{card_id}");

        let mut person = GpeoplePerson::from_vcard(&vcard)?;
        person.resource_name = resource_name.clone();

        let current = self
            .inner
            .person_get(&resource_name, GPEOPLE_PERSON_VCARD_FIELDS, &[])?
            .response;

        let base = current.to_vcard();
        let base_person = GpeoplePerson::from_vcard(&base)?;
        let outcome = CardUpdateOutcome {
            kept_properties: person.unremovable_properties(&base_person),
        };

        let fields = person.changed_fields(&base_person);
        if fields.is_empty() {
            return Ok(outcome);
        }

        // NOTE: a masked update replaces the whole clientData list and
        // other clients may own entries there, so a stash write merges
        // the server's foreign entries first; the etag guard turns a
        // lost-update race into a clean rejection.
        if fields.contains(&GpeoplePersonField::ClientData) {
            let mut merged: Vec<_> = current
                .client_data
                .into_iter()
                .filter(|entry| entry.key.as_deref() != Some(GPEOPLE_PERSON_STASH_KEY))
                .collect();
            merged.append(&mut person.client_data);
            person.client_data = merged;
        }

        person.etag = match if_match {
            Some(etag) => etag.to_string(),
            None => current.etag,
        };

        self.inner
            .contact_update(&person, &fields, GPEOPLE_PERSON_VCARD_FIELDS, &[])?;

        Ok(outcome)
    }

    /// Deletes the contact `card_id`.
    pub fn delete_card(
        &mut self,
        _addressbook_id: &str,
        card_id: &str,
        if_match: Option<&str>,
    ) -> Result<()> {
        if if_match.is_some() {
            bail!("Google People cannot gate a delete on an ETag");
        }
        self.inner.contact_delete(&format!("people/{card_id}"))?;
        Ok(())
    }
}

/// Whether the person is a member of the contact group `id`.
fn in_group(person: &GpeoplePerson, id: &str) -> bool {
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

/// The io-gpeople person as a shared card.
///
/// The projected vCard document is the contents, the person id the id
/// and the person etag the ETag.
fn into_card(addressbook_id: &str, person: GpeoplePerson) -> Card {
    let vcard = person.to_vcard();
    let etag = (!person.etag.is_empty()).then(|| person.etag.clone());

    Card {
        id: person.id().to_string(),
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
