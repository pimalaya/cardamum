//! # Google People backend
//!
//! The People arm of the unified-API client: thin glue mapping the unified
//! addressbook and card operations onto
//! [`io_gpeople::v1::client::GpeopleClientStd`] calls, projecting persons
//! onto vCard documents through io-gpeople's `vcard` feature.
//!
//! Contact groups are the addressbooks: the myContacts system group, the
//! one every contact belongs to, comes first as Contacts, then the
//! user's own groups. Memberships are m:n labels, so one card can appear
//! under several books and each listing is narrowed to its group.

use anyhow::{Error, Result, bail};
use io_gpeople::v1::{
    client::{GpeopleClientStd, GpeopleClientStdConnectOptions, GpeopleClientStdError},
    rest::{
        contact_groups::{
            GpeopleContactGroup, GpeopleContactGroupType, list::GpeopleContactGroupsListParams,
        },
        people::{
            GpeoplePerson,
            connections::list::GpeopleConnectionsListParams,
            vcard::{GPEOPLE_PERSON_VCARD_FIELDS, GpeoplePersonVcard},
        },
    },
};
use log::warn;
use pimalaya_config::secret::SecretResolver;
use secrecy::ExposeSecret;

use crate::{
    config::{GpeopleConfig, ProxyConfig},
    error,
    gpeople::client::is_missing,
    shared::{
        addressbook::{Addressbook, AddressbookDiff},
        card::{Card, CardUpdateOutcome},
        client::window,
    },
};

/// Contact group id of myContacts, the group every contact belongs to.
pub const MY_CONTACTS_GROUP: &str = "myContacts";

/// Google People backend of the unified-API client.
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
        skip: usize,
        take: Option<usize>,
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

        Ok(window(cards, skip, take))
    }

    /// Reads the contact `card_id`, projected onto a vCard document.
    pub fn get_card(&mut self, addressbook_id: &str, card_id: &str) -> Result<Card> {
        let person = self
            .inner
            .person_get(
                &format!("people/{card_id}"),
                GPEOPLE_PERSON_VCARD_FIELDS,
                &[],
            )
            .map_err(|err| card_error(card_id, err))?
            .response;

        Ok(into_card(addressbook_id, person))
    }

    /// Creates the vCard as a People contact, returning its new id.
    ///
    /// Creates always land in myContacts, so a user group target adds
    /// the membership right after. A property too long for the stash
    /// is left out and logged as a warning.
    pub fn create_card(&mut self, addressbook_id: &str, contents: Vec<u8>) -> Result<String> {
        let vcard = into_vcard_text(contents)?;
        let GpeoplePersonVcard { person, dropped } = GpeoplePerson::from_vcard(&vcard)?;

        let created = self
            .inner
            .contact_create(&person, GPEOPLE_PERSON_VCARD_FIELDS, &[])?
            .response;
        let id = created.id().to_string();

        if !dropped.is_empty() {
            let names = dropped.join(", ");
            warn!("card `{id}` stored without {names}: too long for the server");
        }

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
    /// whose etag guards the write unless `if_match` gives one, each
    /// masked field keeping what a vCard cannot show. The returned
    /// properties are stash entries Google refuses to drop, and a
    /// property too long for the stash is left out and logged as a
    /// warning.
    pub fn update_card(
        &mut self,
        _addressbook_id: &str,
        card_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
    ) -> Result<CardUpdateOutcome> {
        let vcard = into_vcard_text(contents)?;
        let resource_name = format!("people/{card_id}");

        let GpeoplePersonVcard {
            mut person,
            dropped,
        } = GpeoplePerson::from_vcard(&vcard)?;
        person.resource_name = resource_name.clone();

        let current = self
            .inner
            .person_get(&resource_name, GPEOPLE_PERSON_VCARD_FIELDS, &[])
            .map_err(|err| card_error(card_id, err))?
            .response;

        let base = current.to_vcard();
        let base_person = GpeoplePerson::from_vcard(&base)?.person;
        let outcome = CardUpdateOutcome {
            kept_properties: person.unremovable_properties(&base_person),
        };

        let fields = person.changed_fields(&base_person);

        if !fields.is_empty() {
            // NOTE: a masked update replaces each field whole; the etag
            // guard turns a lost-update race into a clean rejection.
            person.merge_unprojected(&current, &fields);

            person.etag = match if_match {
                Some(etag) => etag.to_string(),
                None => current.etag,
            };

            self.inner
                .contact_update(&person, &fields, GPEOPLE_PERSON_VCARD_FIELDS, &[])?;
        }

        if !dropped.is_empty() {
            let names = dropped.join(", ");
            warn!("card `{card_id}` stored without {names}: too long for the server");
        }

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
        self.inner
            .contact_delete(&format!("people/{card_id}"))
            .map_err(|err| card_error(card_id, err))?;
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

/// The failure of a request on the person `card_id`, `card-not-found` when
/// People holds no such person.
fn card_error(card_id: &str, err: GpeopleClientStdError) -> Error {
    match is_missing(&err) {
        true => error::card_not_found(card_id, err.into()),
        false => err.into(),
    }
}

/// Decodes raw card bytes as UTF-8 vCard text.
fn into_vcard_text(contents: Vec<u8>) -> Result<String> {
    String::from_utf8(contents).map_err(|_| anyhow::anyhow!("Card contents are not valid UTF-8"))
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use io_gpeople::v1::{client::GpeopleClientStd, rest::people::GpeoplePerson};
    use log::{LevelFilter, Log, Metadata, Record};
    use pimalaya_cli::error::code_of;
    use serde_json::{Value, json};

    use crate::{
        error::{self, ErrorCode},
        gpeople::backend::{GpeopleBackend, MY_CONTACTS_GROUP},
        script::{self, Script},
    };

    static WARNINGS: Warnings = Warnings(Mutex::new(Vec::new()));

    /// A logger keeping the warnings of the backend.
    struct Warnings(Mutex<Vec<String>>);

    impl Log for Warnings {
        fn enabled(&self, metadata: &Metadata) -> bool {
            metadata.target().ends_with("gpeople::backend")
        }

        fn log(&self, record: &Record) {
            if self.enabled(record.metadata()) {
                self.0.lock().unwrap().push(record.args().to_string());
            }
        }

        fn flush(&self) {}
    }

    fn response(body: &Value) -> Vec<u8> {
        script::response(200, "application/json", &body.to_string())
    }

    #[test]
    fn a_create_warns_of_the_properties_too_long_to_store() {
        log::set_logger(&WARNINGS).unwrap();
        log::set_max_level(LevelFilter::Warn);

        let photo = "A".repeat(9000);
        let vcard = format!(
            "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane Doe\r\nX-PET:Rex\r\n\
             PHOTO:data:image/png;base64,{photo}\r\nEND:VCARD\r\n"
        );

        let created = json!({"resourceName": "people/c2", "etag": "%EgUBAi43PRoEAQIFByIM"});
        let (script, requests) = Script::new([response(&created)]);
        let mut backend = GpeopleBackend {
            inner: GpeopleClientStd::new(script, "token"),
        };

        let id = backend
            .create_card(MY_CONTACTS_GROUP, vcard.into_bytes())
            .unwrap();
        assert_eq!(id, "c2");

        let requests = requests.lock().unwrap();
        let create = String::from_utf8(requests[0].clone()).unwrap();
        assert!(create.contains("X-PET:Rex") && !create.contains(&photo));

        let warnings = WARNINGS.0.lock().unwrap();
        assert_eq!(
            *warnings,
            ["card `c2` stored without PHOTO: too long for the server"]
        );
    }

    #[test]
    fn an_update_keeps_what_the_vcard_cannot_show() {
        let current = json!({
            "resourceName": "people/c1",
            "etag": "%EgUBAi43PRoEAQIFByIM",
            "names": [{"givenName": "Jane", "familyName": "Doe"}],
            "organizations": [
                {"name": "Acme", "title": "Engineer", "symbol": "ACM"},
                {"name": "Initech", "title": "Advisor"},
            ],
            "clientData": [
                {"key": "cardamum.vcard", "value": "UID:urn:uuid:4fbe8971"},
                {"key": "other.client", "value": "theirs"},
            ],
        });

        let person: GpeoplePerson = serde_json::from_value(current.clone()).unwrap();
        let vcard = person.to_vcard().replace("Acme", "Globex");
        let vcard = vcard.replace("END:VCARD", "X-PET:Rex\r\nEND:VCARD");

        let (script, requests) = Script::new([response(&current), response(&current)]);
        let mut backend = GpeopleBackend {
            inner: GpeopleClientStd::new(script, "token"),
        };

        let outcome = backend
            .update_card("myContacts", "c1", vcard.into_bytes(), None)
            .unwrap();
        assert_eq!(outcome.kept_properties, Vec::<String>::new());

        let requests = requests.lock().unwrap();
        assert_eq!(requests.len(), 2);

        let update = String::from_utf8(requests[1].clone()).unwrap();
        let (head, body) = update.split_once("\r\n\r\n").unwrap();
        let line = head.lines().next().unwrap();
        assert!(
            line.starts_with("PATCH /v1/people/c1:updateContact?")
                && line.contains("updatePersonFields=clientData%2Corganizations&"),
            "{line}"
        );

        let body: Value = serde_json::from_str(body).unwrap();
        assert_eq!(body["etag"], current["etag"]);
        assert_eq!(
            body["organizations"],
            json!([
                {"name": "Globex", "title": "Engineer", "symbol": "ACM"},
                {"name": "Initech", "title": "Advisor"},
            ])
        );
        assert_eq!(
            body["clientData"],
            json!([
                {"key": "cardamum.vcard", "value": "UID:urn:uuid:4fbe8971\nX-PET:Rex"},
                {"key": "other.client", "value": "theirs"},
            ])
        );
    }

    /// The backend over a People server answering `body` with `status`.
    fn refusing(status: u16, body: &Value) -> GpeopleBackend {
        let answer = script::response(status, "application/json", &body.to_string());
        let (script, _) = Script::new([answer]);
        GpeopleBackend {
            inner: GpeopleClientStd::new(script, "token"),
        }
    }

    /// The code a delete of `c1` refused with `status` and `body` prints
    /// in its failure object.
    fn delete_code(status: u16, body: Value) -> Option<ErrorCode> {
        let err = refusing(status, &body)
            .delete_card(MY_CONTACTS_GROUP, "c1", None)
            .unwrap_err();
        let code = error::code(&err);
        assert_eq!(code_of(&error::with_code(err)), code);
        code
    }

    /// A 403 is a matter of rights unless it names a rate limit or a
    /// quota, which a 429 always is.
    #[test]
    fn a_people_refusal_carries_its_code() {
        let denied = json!({"error": {
            "code": 403,
            "message": "The caller does not have permission",
            "status": "PERMISSION_DENIED",
        }});
        let throttled = json!({"error": {
            "code": 429,
            "message": "Quota exceeded for quota metric 'Read requests'",
            "status": "RESOURCE_EXHAUSTED",
        }});
        let limited = json!({"error": {
            "code": 403,
            "message": "User Rate Limit Exceeded",
            "errors": [{"reason": "userRateLimitExceeded"}],
        }});
        let failed = json!({"error": {"code": 500, "message": "Internal error"}});

        assert_eq!(delete_code(403, denied), Some(ErrorCode::PermissionDenied));
        assert_eq!(delete_code(429, throttled), Some(ErrorCode::RateLimited));
        assert_eq!(delete_code(403, limited), Some(ErrorCode::RateLimited));
        assert_eq!(delete_code(500, failed), None);
    }

    /// A person People does not hold is a card not found, a group not.
    #[test]
    fn a_missing_person_is_a_card_not_found() {
        let missing = json!({"error": {
            "code": 404,
            "message": "Requested entity was not found.",
            "status": "NOT_FOUND",
        }});

        let err = refusing(404, &missing)
            .get_card(MY_CONTACTS_GROUP, "c9")
            .unwrap_err();
        assert_eq!(code_of(&err), Some(ErrorCode::CardNotFound), "{err:#}");
        assert!(format!("{err:#}").contains("Requested entity"), "{err:#}");

        let err = refusing(404, &missing)
            .delete_addressbook("g9")
            .unwrap_err();
        assert_eq!(error::code(&err), None, "{err:#}");
    }
}
