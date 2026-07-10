//! Google People person to vCard projection, and back.
//!
//! The People API exposes no vCard representation of a contact (the
//! people endpoints only speak the JSON person resource), so the Google
//! spoke synthesizes the vCard document of record itself: [`to_vcard`]
//! projects an io-google-people person onto a fresh vCard 4.0 document,
//! and [`to_person`] projects a vCard back onto a person. Per the
//! custom property policy of docs/contacts-mapping.md only fields with
//! a well-defined vCard slot are projected; People-only fields
//! (fileAses, memberships, events, photos, ...) stay out of
//! [`MANAGED_FIELDS`], out of every update mask, and survive updates
//! untouched. Unlike Graph's fixed slots, People fields are true
//! lists, so every vCard property projects without truncation.

use core::str::FromStr;
use std::borrow::Cow;

use io_google_people::v1::rest::people::{
    PeopleAddress, PeopleBiography, PeopleBirthday, PeopleClientData, PeopleContentType,
    PeopleDate, PeopleEmailAddress, PeopleImClient, PeopleName, PeopleNickname, PeopleOccupation,
    PeopleOrganization, PeoplePerson, PeoplePersonField, PeoplePhoneNumber, PeopleRelation,
    PeopleUrl,
};
use vcard::{
    param::VcardParam,
    prop::{VcardProp, VcardPropKind, VcardPropName},
    tree::{
        cst::VcardCst,
        line::VcardLine,
        prop::{
            VcardPropLens, adr::ADR, email::EMAIL, r#fn::FN, impp::IMPP, n::N, nickname::NICKNAME,
            note::NOTE, org::ORG, related::RELATED, role::ROLE, tel::TEL, title::TITLE, url::URL,
        },
    },
    value::{
        VcardValue,
        adr::VcardAdr,
        datetime::VcardDateAndOrTime,
        n::VcardN,
        org::VcardOrg,
        text::{VcardText, VcardTextList},
        uri::VcardUri,
    },
};

use crate::project::{MAX_STASH_LINE, escape_text, full_date, splice_props, text_prop};

/// Person fields the projection reads: the managed set plus the
/// Google-scoped fields minted as X-GOOGLE-* properties, read-only
/// projections that stay out of every update mask.
pub const READ_FIELDS: &[PeoplePersonField] = &[
    PeoplePersonField::Addresses,
    PeoplePersonField::Biographies,
    PeoplePersonField::Birthdays,
    PeoplePersonField::ClientData,
    PeoplePersonField::EmailAddresses,
    PeoplePersonField::ExternalIds,
    PeoplePersonField::ImClients,
    PeoplePersonField::Locations,
    PeoplePersonField::Memberships,
    PeoplePersonField::MiscKeywords,
    PeoplePersonField::Names,
    PeoplePersonField::Nicknames,
    PeoplePersonField::Occupations,
    PeoplePersonField::Organizations,
    PeoplePersonField::PhoneNumbers,
    PeoplePersonField::Relations,
    PeoplePersonField::Urls,
];

/// clientData key of the entry stashing the vCard remainder: every
/// property the projection neither manages nor mints, preserved
/// verbatim through Google (docs/custom-data.md).
pub const CLIENT_DATA_KEY: &str = "cardamum.vcard";

/// Property names minted by [`to_vcard`] from the Google-scoped person
/// fields; [`to_person`] consumes (drops) them, the server value being
/// authoritative. X-GOOGLE-MEMBERSHIP is no longer minted (memberships
/// became structural addressbook data) but stays consumed, so lines
/// from earlier projections are dropped rather than stashed.
const MINTED_PROPS: &[&str] = &[
    "X-GOOGLE-MEMBERSHIP",
    "X-GOOGLE-EXTERNAL-ID",
    "X-GOOGLE-MISC-KEYWORD",
    "X-GOOGLE-LOCATION",
];

/// The bare person id behind a `people/<id>` resource name, the
/// JNI-facing addressing key.
pub fn person_id(resource_name: &str) -> &str {
    resource_name
        .strip_prefix("people/")
        .unwrap_or(resource_name)
}

/// Projects an io-google-people person onto a fresh vCard 4.0 document.
///
/// The person id (resource name minus the `people/` prefix) becomes the
/// UID, typed fields carry their home/work TYPE (phones also mobile as
/// cell), and spouse and children relations become RELATED names.
pub fn to_vcard(person: &PeoplePerson) -> String {
    let mut card = VcardCst::v4();

    let id = person_id(&person.resource_name);
    if !id.is_empty() {
        card.push(text_prop(VcardPropKind::Uid, vec![], id));
    }

    // FN is mandatory: the display name, or composed from the parts.
    card.push(text_prop(VcardPropKind::Fn, vec![], &display_name(person)));

    if let Some(name) = person.names.first() {
        let n = VcardN {
            family: component(&name.family_name),
            given: component(&name.given_name),
            additional: component(&name.middle_name),
            prefixes: component(&name.honorific_prefix),
            suffixes: component(&name.honorific_suffix),
        };

        let empty = n.family.is_empty()
            && n.given.is_empty()
            && n.additional.is_empty()
            && n.prefixes.is_empty()
            && n.suffixes.is_empty();
        if !empty {
            card.push(VcardProp {
                name: VcardPropName::Kind(VcardPropKind::N),
                params: vec![],
                value: VcardValue::N(n),
            });
        }
    }

    for nickname in &person.nicknames {
        if let Some(nick) = opt(&nickname.value) {
            card.push(VcardProp {
                name: VcardPropName::Kind(VcardPropKind::Nickname),
                params: vec![],
                value: VcardValue::TextList(VcardTextList(vec![Cow::Owned(nick.to_string())])),
            });
        }
    }

    for email in &person.email_addresses {
        if let Some(address) = opt(&email.value) {
            let params = std_type(&email.email_type).map(type_param).into_iter();
            card.push(text_prop(VcardPropKind::Email, params.collect(), address));
        }
    }

    for im in &person.im_clients {
        if let Some(username) = opt(&im.username) {
            let uri = match opt(&im.protocol) {
                Some(protocol) => format!("{protocol}:{username}"),
                None => username.to_string(),
            };
            card.push(VcardProp {
                name: VcardPropName::Kind(VcardPropKind::Impp),
                params: vec![],
                value: VcardValue::Uri(VcardUri(Cow::Owned(uri))),
            });
        }
    }

    for phone in &person.phone_numbers {
        if let Some(number) = opt(&phone.value) {
            let params = tel_type(&phone.phone_type).map(type_param).into_iter();
            card.push(text_prop(VcardPropKind::Tel, params.collect(), number));
        }
    }

    for address in &person.addresses {
        if let Some(prop) = adr_prop(address) {
            card.push(prop);
        }
    }

    if let Some(org) = person.organizations.first() {
        let company = opt(&org.name);
        let department = opt(&org.department);
        if company.is_some() || department.is_some() {
            let mut components: Vec<Cow<'static, str>> =
                vec![Cow::Owned(company.unwrap_or_default().to_string())];
            if let Some(department) = department {
                components.push(Cow::Owned(department.to_string()));
            }
            card.push(VcardProp {
                name: VcardPropName::Kind(VcardPropKind::Org),
                params: vec![],
                value: VcardValue::Org(VcardOrg(components)),
            });
        }

        if let Some(title) = opt(&org.title) {
            card.push(text_prop(VcardPropKind::Title, vec![], title));
        }
    }

    if let Some(role) = person.occupations.first().and_then(|role| opt(&role.value)) {
        card.push(text_prop(VcardPropKind::Role, vec![], role));
    }

    for url in &person.urls {
        if let Some(page) = opt(&url.value) {
            card.push(VcardProp {
                name: VcardPropName::Kind(VcardPropKind::Url),
                params: vec![],
                value: VcardValue::Uri(VcardUri(Cow::Owned(page.to_string()))),
            });
        }
    }

    // NOTE: People dates can be partial (year-less birthdays); only a
    // full date has a portable vCard slot, see project::full_date.
    if let Some(date) = person.birthdays.first().and_then(|birthday| {
        let date = birthday.date.as_ref()?;
        Some(format!(
            "{:04}-{:02}-{:02}",
            date.year?, date.month?, date.day?
        ))
    }) {
        card.push(VcardProp {
            name: VcardPropName::Kind(VcardPropKind::Bday),
            params: vec![],
            value: VcardValue::DateAndOrTime(VcardDateAndOrTime(Cow::Owned(date))),
        });
    }

    // NOTE: HTML biographies come from Google profiles, not contacts;
    // they have no plain-text slot and are skipped rather than mangled.
    if let Some(notes) = person
        .biographies
        .iter()
        .find(|bio| bio.content_type != Some(PeopleContentType::TextHtml))
        .and_then(|bio| opt(&bio.value))
    {
        card.push(text_prop(VcardPropKind::Note, vec![], notes));
    }

    for relation in &person.relations {
        if let Some(name) = opt(&relation.person) {
            match opt(&relation.relation_type) {
                Some("spouse") => {
                    card.push(related_prop("spouse", name));
                }
                Some("child") => {
                    card.push(related_prop("child", name));
                }
                _ => {}
            }
        }
    }

    let vcard = String::from_utf8_lossy(&card.to_bytes()).into_owned();

    let mut extra = minted_props(person);
    extra.extend(stash_lines(person));
    splice_props(vcard, &extra)
}

/// Projects a vCard onto an io-google-people person, the full-state
/// projection: every managed field carries the vCard's values (empty
/// when the vCard drops the property, which clears the masked field on
/// update), while unmanaged People fields stay out of the body. Every
/// line that does not project (unknown and X-* properties, standard
/// properties without a People slot, values past a single-instance
/// slot, partial birthdays) is stashed verbatim into the cardamum
/// clientData entry, so it survives on Google and restores on read.
/// The UID is not read back (the resource name addresses the person
/// through the request path, filled by the caller) and the minted
/// X-GOOGLE-* properties are consumed, the server value being
/// authoritative.
pub fn to_person(vcard: &str) -> Result<PeoplePerson, String> {
    let card = VcardCst::parse(vcard).map_err(|err| format!("Invalid vCard: {err}"))?;
    let version = card.version();

    let mut person = PeoplePerson::default();
    let mut name = PeopleName::default();
    let mut org = PeopleOrganization::default();
    let mut notes = Vec::new();
    let mut stash = Vec::new();
    let mut name_seen = false;

    for line in &card.props {
        let consumed = match VcardPropKind::from_str(line.name.get()) {
            // NOTE: the VERSION line is structural and the minted
            // X-GOOGLE-* properties are read-only projections; neither
            // belongs to the remainder.
            Err(_) => {
                let raw_name = line.name.get();
                raw_name.eq_ignore_ascii_case("VERSION")
                    || MINTED_PROPS
                        .iter()
                        .any(|prop| raw_name.eq_ignore_ascii_case(prop))
            }
            // NOTE: the UID is managed: the resource name addresses
            // the person through the request path.
            Ok(VcardPropKind::Uid) => true,
            Ok(VcardPropKind::Fn) => {
                let value = FN::decode(line, version);
                set_first(&mut name.unstructured_name, &value.0)
            }
            Ok(VcardPropKind::N) => {
                if !name_seen {
                    name_seen = true;
                    let n = N::decode(line, version);
                    set_first(&mut name.family_name, n.family.join(" "));
                    set_first(&mut name.given_name, n.given.join(" "));
                    set_first(&mut name.middle_name, n.additional.join(" "));
                    set_first(&mut name.honorific_prefix, n.prefixes.join(" "));
                    set_first(&mut name.honorific_suffix, n.suffixes.join(" "));
                    true
                } else {
                    false
                }
            }
            Ok(VcardPropKind::Nickname) => {
                let all = NICKNAME::decode(line, version);
                let mut pushed = false;
                for nick in &all.0 {
                    let nick = nick.trim();
                    if !nick.is_empty() {
                        person.nicknames.push(PeopleNickname {
                            value: Some(nick.to_string()),
                            ..Default::default()
                        });
                        pushed = true;
                    }
                }
                pushed
            }
            Ok(VcardPropKind::Email) => {
                let email = EMAIL::decode(line, version);
                let address = email.0.trim();
                if !address.is_empty() {
                    person.email_addresses.push(PeopleEmailAddress {
                        value: Some(address.to_string()),
                        email_type: std_type_of(&type_values(line)),
                        ..Default::default()
                    });
                    true
                } else {
                    false
                }
            }
            Ok(VcardPropKind::Impp) => {
                let impp = IMPP::decode(line, version);
                let address = impp.0.trim();
                if !address.is_empty() {
                    let (protocol, username) = match address.split_once(':') {
                        Some((protocol, username)) => (Some(protocol.to_string()), username),
                        None => (None, address),
                    };
                    person.im_clients.push(PeopleImClient {
                        username: Some(username.to_string()),
                        protocol,
                        ..Default::default()
                    });
                    true
                } else {
                    false
                }
            }
            Ok(VcardPropKind::Tel) => {
                let tel = TEL::decode(line, version);
                let number = tel.0.trim();
                if !number.is_empty() {
                    person.phone_numbers.push(PeoplePhoneNumber {
                        value: Some(number.to_string()),
                        phone_type: tel_type_of(&type_values(line)),
                        ..Default::default()
                    });
                    true
                } else {
                    false
                }
            }
            Ok(VcardPropKind::Adr) => {
                let adr = ADR::decode(line, version);

                let address = PeopleAddress {
                    po_box: joined(&adr.po_box),
                    extended_address: joined(&adr.extended),
                    // NOTE: People's street is one multiline field, so
                    // each street component becomes one of its lines.
                    street_address: {
                        let street: Vec<&str> = adr
                            .street
                            .iter()
                            .map(|component| component.as_ref().trim())
                            .filter(|component| !component.is_empty())
                            .collect();
                        (!street.is_empty()).then(|| street.join("\n"))
                    },
                    city: joined(&adr.locality),
                    region: joined(&adr.region),
                    postal_code: joined(&adr.postal_code),
                    country: joined(&adr.country),
                    address_type: std_type_of(&type_values(line)),
                    ..Default::default()
                };

                let empty = address.po_box.is_none()
                    && address.extended_address.is_none()
                    && address.street_address.is_none()
                    && address.city.is_none()
                    && address.region.is_none()
                    && address.postal_code.is_none()
                    && address.country.is_none();
                if !empty {
                    person.addresses.push(address);
                    true
                } else {
                    false
                }
            }
            Ok(VcardPropKind::Org) => {
                if org.name.is_none() && org.department.is_none() {
                    let value = ORG::decode(line, version);
                    let mut components = value.0.iter().map(|component| component.as_ref());
                    let company = components.next().unwrap_or_default();
                    let rest = components.collect::<Vec<_>>().join(" ");
                    set_first(&mut org.name, company);
                    set_first(&mut org.department, rest);
                    true
                } else {
                    false
                }
            }
            Ok(VcardPropKind::Title) => {
                let value = TITLE::decode(line, version);
                set_first(&mut org.title, &value.0)
            }
            Ok(VcardPropKind::Role) => {
                let role = ROLE::decode(line, version);
                let role = role.0.trim();
                if person.occupations.is_empty() && !role.is_empty() {
                    person.occupations.push(PeopleOccupation {
                        value: Some(role.to_string()),
                        ..Default::default()
                    });
                    true
                } else {
                    false
                }
            }
            Ok(VcardPropKind::Url) => {
                let url = URL::decode(line, version);
                let page = url.0.trim();
                if !page.is_empty() {
                    person.urls.push(PeopleUrl {
                        value: Some(page.to_string()),
                        ..Default::default()
                    });
                    true
                } else {
                    false
                }
            }
            Ok(VcardPropKind::Bday) => {
                // NOTE: a partial (year-less) birthday has no People
                // date it round-trips through; it lands in the stash.
                if person.birthdays.is_empty()
                    && let Some(date) = full_date(&line.raw_value_str())
                {
                    let mut parts = date.split('-').map(|part| part.parse().ok());
                    person.birthdays.push(PeopleBirthday {
                        date: Some(PeopleDate {
                            year: parts.next().flatten(),
                            month: parts.next().flatten(),
                            day: parts.next().flatten(),
                        }),
                        ..Default::default()
                    });
                    true
                } else {
                    false
                }
            }
            Ok(VcardPropKind::Note) => {
                let note = NOTE::decode(line, version);
                let note = note.0.trim();
                if !note.is_empty() {
                    notes.push(note.to_string());
                    true
                } else {
                    false
                }
            }
            Ok(VcardPropKind::Related) => {
                // NOTE: only free-form spouse and child names
                // (VALUE=text) project; other types and URI RELATED
                // lines land in the stash.
                let text = line.params.iter().any(|param| {
                    param.name.get().eq_ignore_ascii_case("VALUE")
                        && param
                            .values
                            .iter()
                            .any(|value| value.get().eq_ignore_ascii_case("text"))
                });
                let related = RELATED::decode(line, version);
                let related = related.0.trim();
                let types = type_values(line);
                let relation_type = if types.iter().any(|t| t == "spouse") {
                    Some("spouse")
                } else if types.iter().any(|t| t == "child") {
                    Some("child")
                } else {
                    None
                };

                if let Some(relation_type) = relation_type
                    && text
                    && !related.is_empty()
                {
                    person.relations.push(PeopleRelation {
                        person: Some(related.to_string()),
                        relation_type: Some(relation_type.to_string()),
                        ..Default::default()
                    });
                    true
                } else {
                    false
                }
            }
            Ok(_) => false,
        };

        if !consumed {
            let raw = line.to_string();
            let raw = raw.trim_end().to_string();
            if raw.len() <= MAX_STASH_LINE {
                stash.push(raw);
            }
        }
    }

    if !stash.is_empty() {
        person.client_data = vec![PeopleClientData {
            key: Some(CLIENT_DATA_KEY.to_string()),
            value: Some(stash.join("\n")),
            ..Default::default()
        }];
    }

    let empty = name.unstructured_name.is_none()
        && name.family_name.is_none()
        && name.given_name.is_none()
        && name.middle_name.is_none()
        && name.honorific_prefix.is_none()
        && name.honorific_suffix.is_none();
    if !empty {
        person.names.push(name);
    }

    if org.name.is_some() || org.department.is_some() || org.title.is_some() {
        person.organizations.push(org);
    }

    if !notes.is_empty() {
        person.biographies.push(PeopleBiography {
            value: Some(notes.join("\n")),
            content_type: Some(PeopleContentType::TextPlain),
            ..Default::default()
        });
    }

    Ok(person)
}

/// The managed fields whose projection differs between the edited
/// person and the base one (the state last synced with the server):
/// the update mask shrinks to them, so unchanged fields are neither
/// replaced nor clobbered by a concurrent edit.
pub fn changed_fields(person: &PeoplePerson, base: &PeoplePerson) -> Vec<PeoplePersonField> {
    let mut fields = Vec::new();

    macro_rules! push_changed {
        ($($field:ident => $variant:ident),* $(,)?) => {$(
            if person.$field != base.$field {
                fields.push(PeoplePersonField::$variant);
            }
        )*};
    }

    push_changed!(
        addresses => Addresses,
        biographies => Biographies,
        birthdays => Birthdays,
        client_data => ClientData,
        email_addresses => EmailAddresses,
        im_clients => ImClients,
        names => Names,
        nicknames => Nicknames,
        occupations => Occupations,
        organizations => Organizations,
        phone_numbers => PhoneNumbers,
        relations => Relations,
        urls => Urls,
    );

    fields
}

/// The person's display name: the server-formatted one, the
/// unstructured name, or composed from the parts.
fn display_name(person: &PeoplePerson) -> String {
    let Some(name) = person.names.first() else {
        return String::new();
    };

    if let Some(display) = opt(&name.display_name) {
        return display.to_string();
    }
    if let Some(unstructured) = opt(&name.unstructured_name) {
        return unstructured.to_string();
    }

    let composed: Vec<&str> = [&name.given_name, &name.middle_name, &name.family_name]
        .into_iter()
        .filter_map(opt)
        .collect();
    composed.join(" ")
}

/// An ADR property from a People address, or None when every component
/// is empty. People's street is one multiline field, so each of its
/// lines becomes a vCard street component.
fn adr_prop(address: &PeopleAddress) -> Option<VcardProp<'static>> {
    let street = address.street_address.as_deref().unwrap_or("");
    let value = VcardAdr {
        po_box: option_component(&address.po_box),
        extended: option_component(&address.extended_address),
        street: street
            .split('\n')
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(|line| Cow::Owned(line.to_string()))
            .collect(),
        locality: option_component(&address.city),
        region: option_component(&address.region),
        postal_code: option_component(&address.postal_code),
        country: option_component(&address.country),
        ..Default::default()
    };

    let empty = value.po_box.is_empty()
        && value.extended.is_empty()
        && value.street.is_empty()
        && value.locality.is_empty()
        && value.region.is_empty()
        && value.postal_code.is_empty()
        && value.country.is_empty();
    if empty {
        return None;
    }

    Some(VcardProp {
        name: VcardPropName::Kind(VcardPropKind::Adr),
        params: std_type(&address.address_type)
            .map(type_param)
            .into_iter()
            .collect(),
        value: VcardValue::Adr(value),
    })
}

/// A RELATED name property (spouse, child): free-form text, hence the
/// explicit VALUE=text (RELATED defaults to a URI).
fn related_prop(r#type: &'static str, name: &str) -> VcardProp<'static> {
    VcardProp {
        name: VcardPropName::Kind(VcardPropKind::Related),
        params: vec![type_param(r#type), VcardParam::Value(Cow::Borrowed("text"))],
        value: VcardValue::Text(VcardText(Cow::Owned(name.to_string()))),
    }
}

/// A single-value TYPE parameter.
fn type_param(value: &'static str) -> VcardParam<'static> {
    VcardParam::Type(vec![Cow::Borrowed(value)])
}

/// The vCard TYPE behind a People home/work field type.
fn std_type(field_type: &Option<String>) -> Option<&'static str> {
    match opt(field_type)? {
        "home" => Some("home"),
        "work" => Some("work"),
        _ => None,
    }
}

/// The People field type behind vCard home/work TYPE values.
fn std_type_of(types: &[String]) -> Option<String> {
    if types.iter().any(|t| t == "home") {
        Some("home".to_string())
    } else if types.iter().any(|t| t == "work") {
        Some("work".to_string())
    } else {
        None
    }
}

/// The vCard TYPE behind a People phone type (mobile maps to cell).
fn tel_type(phone_type: &Option<String>) -> Option<&'static str> {
    match opt(phone_type)? {
        "mobile" => Some("cell"),
        "home" => Some("home"),
        "work" => Some("work"),
        _ => None,
    }
}

/// The People phone type behind vCard TYPE values (cell maps to mobile).
fn tel_type_of(types: &[String]) -> Option<String> {
    if types.iter().any(|t| t == "cell") {
        Some("mobile".to_string())
    } else if types.iter().any(|t| t == "home") {
        Some("home".to_string())
    } else if types.iter().any(|t| t == "work") {
        Some("work".to_string())
    } else {
        None
    }
}

/// The trimmed field, None when unset or blank.
fn opt(field: &Option<String>) -> Option<&str> {
    let value = field.as_deref()?.trim();
    if value.is_empty() { None } else { Some(value) }
}

/// A structured-value component holding the trimmed field, or empty.
fn component(field: &Option<String>) -> Vec<Cow<'static, str>> {
    match opt(field) {
        Some(value) => vec![Cow::Owned(value.to_string())],
        None => Vec::new(),
    }
}

/// Like [`component`], in the People to vCard direction.
fn option_component(field: &Option<String>) -> Vec<Cow<'static, str>> {
    component(field)
}

/// Fills a single-instance People field, first non-empty value wins;
/// true when this value took the slot.
fn set_first(slot: &mut Option<String>, value: impl AsRef<str>) -> bool {
    let value = value.as_ref().trim();
    if slot.is_none() && !value.is_empty() {
        *slot = Some(value.to_string());
        true
    } else {
        false
    }
}

/// X-GOOGLE-* lines minted from the Google-scoped person fields the
/// projection exposes read-only: external ids, miscellaneous keywords
/// and locations mean nothing outside the account they came from, so
/// they ride the vCard as vendor properties (docs/custom-data.md) and
/// are consumed on the way back. Group memberships are NOT minted:
/// they are the card's addressbook memberships (docs/merged-view.md),
/// surfaced structurally on the JNI card instead.
fn minted_props(person: &PeoplePerson) -> Vec<String> {
    let mut lines = Vec::new();

    for id in &person.external_ids {
        if let Some(value) = opt(&id.value) {
            lines.push(typed_line("X-GOOGLE-EXTERNAL-ID", &id.id_type, value));
        }
    }

    for keyword in &person.misc_keywords {
        if let Some(value) = opt(&keyword.value) {
            // NOTE: the serde wire name of the keyword type enum is the
            // canonical People spelling (HOME, OUTLOOK_KEYWORD, ...).
            let keyword_type = keyword
                .keyword_type
                .and_then(|t| serde_json::to_value(t).ok())
                .and_then(|v| v.as_str().map(str::to_string));
            lines.push(typed_line("X-GOOGLE-MISC-KEYWORD", &keyword_type, value));
        }
    }

    for location in &person.locations {
        if let Some(value) = opt(&location.value) {
            lines.push(typed_line(
                "X-GOOGLE-LOCATION",
                &location.location_type,
                value,
            ));
        }
    }

    lines
}

/// A minted line with an optional TYPE parameter, carried only when
/// the type is a plain token a parameter value holds unquoted.
fn typed_line(name: &str, r#type: &Option<String>, value: &str) -> String {
    let value = escape_text(value);
    let token = opt(r#type).filter(|t| {
        t.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    });

    match token {
        Some(token) => format!("{name};TYPE={token}:{value}"),
        None => format!("{name}:{value}"),
    }
}

/// The stashed vCard remainder lines behind the person's cardamum
/// clientData entry.
fn stash_lines(person: &PeoplePerson) -> Vec<String> {
    person
        .client_data
        .iter()
        .filter(|entry| entry.key.as_deref() == Some(CLIENT_DATA_KEY))
        .filter_map(|entry| entry.value.as_deref())
        .flat_map(|value| value.split('\n'))
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

/// The structured-value components joined into one People field, None
/// when nothing remains.
fn joined(components: &[Cow<'_, str>]) -> Option<String> {
    let joined = components
        .iter()
        .map(|component| component.as_ref().trim())
        .filter(|component| !component.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    if joined.is_empty() {
        None
    } else {
        Some(joined)
    }
}

/// Collects every TYPE parameter value, lowercased.
fn type_values(line: &VcardLine) -> Vec<String> {
    let mut types = Vec::new();
    for param in &line.params {
        if param.name.get().eq_ignore_ascii_case("TYPE") {
            types.extend(
                param
                    .values
                    .iter()
                    .map(|value| value.get().to_ascii_lowercase()),
            );
        }
    }
    types
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A person filling every field the projection manages.
    fn full_person() -> PeoplePerson {
        PeoplePerson {
            names: vec![PeopleName {
                unstructured_name: Some("Jane Doe".into()),
                family_name: Some("Doe".into()),
                given_name: Some("Jane".into()),
                middle_name: Some("Q.".into()),
                honorific_prefix: Some("Dr.".into()),
                honorific_suffix: Some("PhD".into()),
                ..Default::default()
            }],
            nicknames: vec![PeopleNickname {
                value: Some("Janie".into()),
                ..Default::default()
            }],
            email_addresses: vec![PeopleEmailAddress {
                value: Some("jane@doe.org".into()),
                email_type: Some("home".into()),
                ..Default::default()
            }],
            im_clients: vec![PeopleImClient {
                username: Some("jane@doe.org".into()),
                protocol: Some("xmpp".into()),
                ..Default::default()
            }],
            phone_numbers: vec![
                PeoplePhoneNumber {
                    value: Some("+331111".into()),
                    phone_type: Some("work".into()),
                    ..Default::default()
                },
                PeoplePhoneNumber {
                    value: Some("+333333".into()),
                    phone_type: Some("mobile".into()),
                    ..Default::default()
                },
            ],
            addresses: vec![PeopleAddress {
                street_address: Some("12 Main St".into()),
                city: Some("Paris".into()),
                region: Some("IDF".into()),
                postal_code: Some("75000".into()),
                country: Some("France".into()),
                address_type: Some("home".into()),
                ..Default::default()
            }],
            organizations: vec![PeopleOrganization {
                name: Some("ACME".into()),
                department: Some("R&D".into()),
                title: Some("Boss".into()),
                ..Default::default()
            }],
            occupations: vec![PeopleOccupation {
                value: Some("Engineer".into()),
                ..Default::default()
            }],
            urls: vec![PeopleUrl {
                value: Some("https://doe.org".into()),
                ..Default::default()
            }],
            birthdays: vec![PeopleBirthday {
                date: Some(PeopleDate {
                    year: Some(1983),
                    month: Some(4),
                    day: Some(1),
                }),
                ..Default::default()
            }],
            biographies: vec![PeopleBiography {
                value: Some("a note".into()),
                content_type: Some(PeopleContentType::TextPlain),
                ..Default::default()
            }],
            relations: vec![
                PeopleRelation {
                    person: Some("John Doe".into()),
                    relation_type: Some("spouse".into()),
                    ..Default::default()
                },
                PeopleRelation {
                    person: Some("Jimmy".into()),
                    relation_type: Some("child".into()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn to_vcard_projects_every_mapped_field() {
        let mut person = full_person();
        person.resource_name = "people/c123".into();

        let vcard = to_vcard(&person);
        assert!(vcard.contains("VERSION:4.0\r\n"));
        assert!(vcard.contains("UID:c123\r\n"));
        assert!(vcard.contains("FN:Jane Doe\r\n"));
        assert!(vcard.contains("N:Doe;Jane;Q.;Dr.;PhD\r\n"));
        assert!(vcard.contains("NICKNAME:Janie\r\n"));
        assert!(vcard.contains("EMAIL;TYPE=home:jane@doe.org\r\n"));
        assert!(vcard.contains("IMPP:xmpp:jane@doe.org\r\n"));
        assert!(vcard.contains("TEL;TYPE=work:+331111\r\n"));
        assert!(vcard.contains("TEL;TYPE=cell:+333333\r\n"));
        assert!(vcard.contains("ADR;TYPE=home:;;12 Main St;Paris;IDF;75000;France\r\n"));
        assert!(vcard.contains("ORG:ACME;R&D\r\n"));
        assert!(vcard.contains("TITLE:Boss\r\n"));
        assert!(vcard.contains("ROLE:Engineer\r\n"));
        assert!(vcard.contains("URL:https://doe.org\r\n"));
        assert!(vcard.contains("BDAY:1983-04-01\r\n"));
        assert!(vcard.contains("NOTE:a note\r\n"));
        assert!(vcard.contains("RELATED;TYPE=spouse;VALUE=text:John Doe\r\n"));
        assert!(vcard.contains("RELATED;TYPE=child;VALUE=text:Jimmy\r\n"));
    }

    #[test]
    fn to_person_reads_managed_props() {
        let vcard = "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:abc\r\nFN:Jane Doe\r\n\
            N:Doe;Jane;Q.;Dr.;PhD\r\nNICKNAME:Janie,JJ\r\n\
            EMAIL;TYPE=home:jane@doe.org\r\nIMPP:xmpp:jane@doe.org\r\n\
            TEL;TYPE=cell:+333333\r\nTEL;TYPE=home,voice:+332222\r\nTEL:+331111\r\n\
            ADR;TYPE=home:box;ext;12 Main St;Paris;IDF;75000;France\r\nORG:ACME;R&D\r\n\
            TITLE:Boss\r\nROLE:Engineer\r\nURL:https://doe.org\r\nBDAY:1983-04-01\r\n\
            NOTE:a note\r\nRELATED;TYPE=spouse;VALUE=text:John Doe\r\n\
            RELATED;TYPE=child;VALUE=text:Jimmy\r\nEND:VCARD\r\n";

        let person = to_person(vcard).unwrap();
        assert_eq!(person.resource_name, "");

        let name = &person.names[0];
        assert_eq!(name.unstructured_name.as_deref(), Some("Jane Doe"));
        assert_eq!(name.family_name.as_deref(), Some("Doe"));
        assert_eq!(name.given_name.as_deref(), Some("Jane"));
        assert_eq!(name.middle_name.as_deref(), Some("Q."));
        assert_eq!(name.honorific_prefix.as_deref(), Some("Dr."));
        assert_eq!(name.honorific_suffix.as_deref(), Some("PhD"));

        assert_eq!(person.nicknames.len(), 2);
        assert_eq!(person.nicknames[0].value.as_deref(), Some("Janie"));

        let email = &person.email_addresses[0];
        assert_eq!(email.value.as_deref(), Some("jane@doe.org"));
        assert_eq!(email.email_type.as_deref(), Some("home"));

        let im = &person.im_clients[0];
        assert_eq!(im.username.as_deref(), Some("jane@doe.org"));
        assert_eq!(im.protocol.as_deref(), Some("xmpp"));

        assert_eq!(person.phone_numbers.len(), 3);
        assert_eq!(
            person.phone_numbers[0].phone_type.as_deref(),
            Some("mobile")
        );
        assert_eq!(person.phone_numbers[1].phone_type.as_deref(), Some("home"));
        assert_eq!(person.phone_numbers[2].phone_type, None);

        let address = &person.addresses[0];
        assert_eq!(address.po_box.as_deref(), Some("box"));
        assert_eq!(address.extended_address.as_deref(), Some("ext"));
        assert_eq!(address.street_address.as_deref(), Some("12 Main St"));
        assert_eq!(address.city.as_deref(), Some("Paris"));
        assert_eq!(address.region.as_deref(), Some("IDF"));
        assert_eq!(address.postal_code.as_deref(), Some("75000"));
        assert_eq!(address.country.as_deref(), Some("France"));
        assert_eq!(address.address_type.as_deref(), Some("home"));

        let org = &person.organizations[0];
        assert_eq!(org.name.as_deref(), Some("ACME"));
        assert_eq!(org.department.as_deref(), Some("R&D"));
        assert_eq!(org.title.as_deref(), Some("Boss"));

        assert_eq!(person.occupations[0].value.as_deref(), Some("Engineer"));
        assert_eq!(person.urls[0].value.as_deref(), Some("https://doe.org"));

        let date = person.birthdays[0].date.as_ref().unwrap();
        assert_eq!(
            (date.year, date.month, date.day),
            (Some(1983), Some(4), Some(1))
        );

        assert_eq!(person.biographies[0].value.as_deref(), Some("a note"));

        assert_eq!(person.relations.len(), 2);
        assert_eq!(person.relations[0].person.as_deref(), Some("John Doe"));
        assert_eq!(person.relations[0].relation_type.as_deref(), Some("spouse"));
    }

    #[test]
    fn changed_fields_shrinks_to_the_edit() {
        let base = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane Doe\r\n\
            TEL;TYPE=cell:+333333\r\nEMAIL:jane@doe.org\r\n\
            NOTE:a note\r\nEND:VCARD\r\n";
        let edited = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane Doe\r\n\
            TEL;TYPE=cell:+444444\r\nEMAIL:jane@doe.org\r\nEND:VCARD\r\n";

        let fields = changed_fields(&to_person(edited).unwrap(), &to_person(base).unwrap());
        assert_eq!(
            fields,
            vec![
                PeoplePersonField::Biographies,
                PeoplePersonField::PhoneNumbers
            ]
        );
    }

    #[test]
    fn changed_fields_of_identical_cards_is_empty() {
        let vcard = to_vcard(&full_person());
        let person = to_person(&vcard).unwrap();
        assert!(changed_fields(&person, &person.clone()).is_empty());
    }

    #[test]
    fn round_trip() {
        let person = full_person();
        assert_eq!(to_person(&to_vcard(&person)).unwrap(), person);
    }

    #[test]
    fn stash_preserves_unprojected_props() {
        let vcard = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:X\r\nX-FOO;TYPE=bar:baz\r\n\
            GENDER:F\r\nBDAY:--0412\r\nEND:VCARD\r\n";

        let person = to_person(vcard).unwrap();
        assert!(person.birthdays.is_empty());

        let stash = &person.client_data[0];
        assert_eq!(stash.key.as_deref(), Some(CLIENT_DATA_KEY));
        assert_eq!(
            stash.value.as_deref(),
            Some("X-FOO;TYPE=bar:baz\nGENDER:F\nBDAY:--0412")
        );

        let restored = to_vcard(&person);
        assert!(restored.contains("X-FOO;TYPE=bar:baz\r\n"));
        assert!(restored.contains("GENDER:F\r\n"));
        assert!(restored.contains("BDAY:--0412\r\n"));
        assert!(restored.ends_with("END:VCARD\r\n"));

        // The restored document projects back to the same person.
        assert_eq!(to_person(&restored).unwrap(), person);
    }

    #[test]
    fn minted_props_project_and_consume() {
        use io_google_people::v1::rest::people::{
            PeopleContactGroupMembership, PeopleExternalId, PeopleMembership,
        };

        let mut person = full_person();
        person.memberships = vec![PeopleMembership {
            contact_group_membership: Some(PeopleContactGroupMembership {
                contact_group_resource_name: Some("contactGroups/myContacts".into()),
                ..Default::default()
            }),
            ..Default::default()
        }];
        person.external_ids = vec![PeopleExternalId {
            value: Some("42".into()),
            id_type: Some("account".into()),
            ..Default::default()
        }];

        // Memberships are structural addressbook data, not a minted
        // vendor property.
        let vcard = to_vcard(&person);
        assert!(!vcard.contains("X-GOOGLE-MEMBERSHIP"));
        assert!(vcard.contains("X-GOOGLE-EXTERNAL-ID;TYPE=account:42\r\n"));

        // Consumed on the way back: the server value is authoritative,
        // so the minted lines neither project nor reach the stash; a
        // legacy X-GOOGLE-MEMBERSHIP line is dropped the same way.
        let legacy = vcard.replace(
            "END:VCARD",
            "X-GOOGLE-MEMBERSHIP:contactGroups/myContacts\r\nEND:VCARD",
        );
        let back = to_person(&legacy).unwrap();
        assert!(back.memberships.is_empty());
        assert!(back.external_ids.is_empty());
        assert!(back.client_data.is_empty());
    }

    #[test]
    fn changed_fields_tracks_the_stash() {
        let base = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:X\r\nEND:VCARD\r\n";
        let edited = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:X\r\nX-FOO:bar\r\nEND:VCARD\r\n";

        let fields = changed_fields(&to_person(edited).unwrap(), &to_person(base).unwrap());
        assert_eq!(fields, vec![PeoplePersonField::ClientData]);
    }

    #[test]
    fn stash_skips_oversized_lines() {
        let photo = format!("PHOTO:data:image/jpeg;base64,{}", "A".repeat(10_000));
        let vcard =
            format!("BEGIN:VCARD\r\nVERSION:4.0\r\nFN:X\r\n{photo}\r\nX-FOO:bar\r\nEND:VCARD\r\n");

        let person = to_person(&vcard).unwrap();
        assert_eq!(person.client_data[0].value.as_deref(), Some("X-FOO:bar"));
    }
}
