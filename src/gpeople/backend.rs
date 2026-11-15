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

use anyhow::{Context, Error, Result, bail};
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
use pimalaya_cli::error::with_code;
use pimalaya_config::secret::SecretResolver;
use secrecy::ExposeSecret;

use crate::{
    config::{GpeopleConfig, ProxyConfig},
    error::{self, ErrorCode},
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
            .contact_group_get(&resource_name, None, &[])
            .map_err(|err| group_error(id, err))?
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
            .contact_group_delete(&format!("contactGroups/{id}"), false)
            .map_err(|err| group_error(id, err))?;
        Ok(())
    }

    /// Fails coded `addressbook-not-found` unless People holds the contact
    /// group `id`, before anything is read or written through it.
    ///
    /// A listing narrows every connection to the group's members and a
    /// create lands in myContacts before its membership is added, so
    /// without this a typo would list as an empty group, and a create
    /// would leave a contact behind before failing. myContacts is every
    /// account's and costs no request; a user group is read once, its
    /// members left out.
    fn resolve_group(&mut self, id: &str) -> Result<()> {
        if id == MY_CONTACTS_GROUP {
            return Ok(());
        }

        self.inner
            .contact_group_get(&format!("contactGroups/{id}"), None, &[])
            .map_err(|err| group_error(id, err))?;
        Ok(())
    }

    /// Lists the contacts of the group, each projected onto a vCard.
    ///
    /// An unknown group fails rather than list as empty.
    pub fn list_cards(
        &mut self,
        addressbook_id: &str,
        skip: usize,
        take: Option<usize>,
    ) -> Result<Vec<Card>> {
        self.resolve_group(addressbook_id)?;

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
        self.resolve_group(addressbook_id)?;

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
    /// the membership right after, the group resolved before the contact
    /// is created: an unknown one creates nothing. A membership refused
    /// after the create says the contact stays in myContacts. A property
    /// too long for the stash is left out and logged as a warning.
    pub fn create_card(&mut self, addressbook_id: &str, contents: Vec<u8>) -> Result<String> {
        let vcard = into_vcard_text(contents)?;
        let GpeoplePersonVcard { person, dropped } = GpeoplePerson::from_vcard(&vcard)?;

        self.resolve_group(addressbook_id)?;

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
            let kept = || {
                format!(
                    "Card `{id}` was created in Contacts but not added to addressbook \
                     `{addressbook_id}`: delete it, or add it to the group"
                )
            };

            let modified = self
                .inner
                .contact_group_members_modify(
                    &format!("contactGroups/{addressbook_id}"),
                    &[created.resource_name],
                    &[],
                )
                .with_context(kept)?
                .response;

            if !modified.not_found_resource_names.is_empty() {
                let names = &modified.not_found_resource_names;
                let err = anyhow::anyhow!("Google group member add rejected: {names:?} not found");
                return Err(err.context(kept()));
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
        addressbook_id: &str,
        card_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
    ) -> Result<CardUpdateOutcome> {
        let vcard = into_vcard_text(contents)?;
        self.resolve_group(addressbook_id)?;
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
        addressbook_id: &str,
        card_id: &str,
        if_match: Option<&str>,
    ) -> Result<()> {
        if if_match.is_some() {
            bail!("Google People cannot gate a delete on an ETag");
        }
        self.resolve_group(addressbook_id)?;
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

/// The failure of a request on the contact group `id`,
/// `addressbook-not-found` when People holds no such group: cardamum's
/// sentence, People's answer in `sources`.
fn group_error(id: &str, err: GpeopleClientStdError) -> Error {
    match is_missing(&err) {
        true => with_code(err.into(), |_| Some(ErrorCode::AddressbookNotFound))
            .context(format!("Addressbook `{id}` not found")),
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
    use pimalaya_cli::error::{code_of, with_code};
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
        assert_eq!(code_of(&with_code(err, error::code)), code);
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

    /// A person People does not hold is a card not found, a group an
    /// addressbook not found.
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
        let code = Some(ErrorCode::AddressbookNotFound);
        assert_eq!(code_of(&err), code, "{err:#}");
        assert_eq!(err.to_string(), "Addressbook `g9` not found");
    }

    /// People's answer to a request on something it does not hold.
    fn not_found() -> Vec<u8> {
        let body = json!({"error": {
            "code": 404,
            "message": "Requested entity was not found.",
            "status": "NOT_FOUND",
        }});
        script::response(404, "application/json", &body.to_string())
    }

    /// The request lines a scripted server received, in order.
    fn lines(requests: &crate::script::Requests) -> Vec<String> {
        let requests = requests.lock().unwrap();
        requests
            .iter()
            .map(|request| {
                let request = String::from_utf8_lossy(request);
                request.lines().next().unwrap_or_default().to_owned()
            })
            .collect()
    }

    const CARD: &str = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane Doe\r\nEND:VCARD\r\n";

    /// A card command run against a backend, its result dropped.
    type Call = fn(&mut GpeopleBackend) -> anyhow::Result<()>;

    /// A card command naming a contact group People does not hold fails
    /// coded `addressbook-not-found` on the one request reading the group:
    /// a create sends no `createContact`, so no contact is left in
    /// myContacts, and a listing does not read as an empty group.
    #[test]
    fn an_unknown_group_is_not_found_before_anything_is_written() {
        let calls: [(&str, Call); 5] = [
            ("create", |b| b.create_card("nope", CARD.into()).map(drop)),
            ("list", |b| b.list_cards("nope", 0, None).map(drop)),
            ("read", |b| b.get_card("nope", "c1").map(drop)),
            ("update", |b| {
                b.update_card("nope", "c1", CARD.into(), None).map(drop)
            }),
            ("delete", |b| b.delete_card("nope", "c1", None)),
        ];

        for (name, call) in calls {
            let (script, requests) = Script::new([not_found()]);
            let mut backend = GpeopleBackend {
                inner: GpeopleClientStd::new(script, "token"),
            };

            let err = call(&mut backend).unwrap_err();
            let code = Some(ErrorCode::AddressbookNotFound);
            assert_eq!(code_of(&err), code, "{name}: {err:#}");
            assert_eq!(err.to_string(), "Addressbook `nope` not found", "{name}");

            let lines = lines(&requests);
            assert_eq!(lines.len(), 1, "{name}: {lines:?}");
            assert!(
                lines[0].starts_with("GET /v1/contactGroups/nope"),
                "{name}: {lines:?}"
            );
        }
    }

    /// A create into a group People holds reads the group, creates the
    /// contact, then adds the membership; a membership refused after the
    /// create says the contact stays in Contacts.
    #[test]
    fn a_create_into_a_group_adds_the_membership_or_says_what_stays() {
        let group = json!({"resourceName": "contactGroups/g1", "etag": "e1", "name": "Work"});
        let created = json!({"resourceName": "people/c2", "etag": "%EgUBAi43PRoEAQIFByIM"});
        let added = json!({});

        let (script, requests) =
            Script::new([response(&group), response(&created), response(&added)]);
        let mut backend = GpeopleBackend {
            inner: GpeopleClientStd::new(script, "token"),
        };
        let id = backend.create_card("g1", CARD.into()).unwrap();
        assert_eq!(id, "c2");

        let lines = lines(&requests);
        assert_eq!(lines.len(), 3, "{lines:?}");
        assert!(
            lines[0].starts_with("GET /v1/contactGroups/g1"),
            "{lines:?}"
        );
        assert!(lines[1].contains("people:createContact"), "{lines:?}");
        assert!(
            lines[2].contains("contactGroups/g1/members:modify"),
            "{lines:?}"
        );

        let (script, _) = Script::new([response(&group), response(&created), not_found()]);
        let mut backend = GpeopleBackend {
            inner: GpeopleClientStd::new(script, "token"),
        };
        let err = backend.create_card("g1", CARD.into()).unwrap_err();
        let error = err.to_string();
        assert!(
            error.starts_with("Card `c2` was created in Contacts"),
            "{error}"
        );
        assert!(error.contains("addressbook `g1`"), "{error}");
    }

    /// myContacts is every account's: a listing of it reads no group.
    #[test]
    fn my_contacts_is_never_resolved() {
        let page = json!({"connections": [], "totalItems": 0});
        let (script, requests) = Script::new([response(&page)]);
        let mut backend = GpeopleBackend {
            inner: GpeopleClientStd::new(script, "token"),
        };

        let cards = backend.list_cards(MY_CONTACTS_GROUP, 0, None).unwrap();
        assert!(cards.is_empty());

        let lines = lines(&requests);
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert!(lines[0].contains("connections"), "{lines:?}");
    }
}
