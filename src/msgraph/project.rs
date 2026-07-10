//! Microsoft Graph contact to vCard projection, and back.
//!
//! Microsoft Graph exposes no vCard representation of a contact (the
//! contacts endpoints only speak the JSON contact resource), so the
//! Graph spoke synthesizes the vCard document of record itself:
//! [`to_vcard`] projects an io-msgraph contact onto a fresh vCard 4.0
//! document, and [`to_contact`] projects a vCard back onto a contact.
//! Per the custom property policy of docs/contacts-mapping.md only
//! fields with a well-defined vCard slot are projected. [`to_contact`]
//! builds a full-state projection: every managed field is Set (or Set
//! empty) when the vCard carries it and Null when it does not;
//! Graph-only fields (fileAs, officeLocation, assistantName, manager)
//! stay Unset, out of the body, and survive updates untouched. Wire
//! bodies refine it: [`to_new_contact`] shapes the create body and
//! [`to_contact_delta`] the update body, both stripping the gratuitous
//! nulls the Outlook backend rejects with HTTP 500.

use core::str::FromStr;
use std::borrow::Cow;

use io_msgraph::v1::{
    MsgraphField,
    rest::users::{
        contacts::{MsgraphContact, MsgraphPhysicalAddress, MsgraphSingleValueExtendedProperty},
        messages::MsgraphEmailAddress,
    },
};
use serde_json::{from_value, to_value};
use vcard::{
    param::VcardParam,
    prop::{VcardProp, VcardPropKind, VcardPropName},
    tree::{
        cst::VcardCst,
        line::VcardLine,
        prop::{
            VcardPropLens, adr::ADR, categories::CATEGORIES, email::EMAIL, r#fn::FN, impp::IMPP,
            n::N, nickname::NICKNAME, note::NOTE, org::ORG, related::RELATED, role::ROLE, tel::TEL,
            title::TITLE, url::URL,
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

/// Extended-property id under which the vCard remainder is stashed on
/// the Graph contact (a fixed app GUID plus a name, the String MAPI
/// form); it rides inline in create and update bodies and reads back
/// through a filtered `$expand` (docs/custom-data.md).
pub const EXTENDED_PROP_ID: &str =
    "String {c8e5e5cf-3f6c-4f0a-9d4e-52f1e7b2a9d3} Name cardamum-vcard";

/// Property names minted by [`to_vcard`] from the Graph-only contact
/// fields; [`to_contact`] consumes (drops) them, the server value
/// being authoritative (the fields stay Unset, out of every body).
const MINTED_PROPS: &[&str] = &[
    "X-MSGRAPH-FILE-AS",
    "X-MSGRAPH-OFFICE-LOCATION",
    "X-MSGRAPH-ASSISTANT",
    "X-MSGRAPH-MANAGER",
];

/// Projects an io-msgraph contact onto a fresh vCard 4.0 document.
///
/// The Graph id becomes the UID, the three phone slots become typed
/// TEL properties (business work, home home, mobile cell), the three
/// address slots become typed ADR properties, and spouse and children
/// become RELATED names (their types are standard in vCard 4.0).
pub fn to_vcard(contact: &MsgraphContact) -> String {
    let mut card = VcardCst::v4();

    let id = contact.id.trim();
    if !id.is_empty() {
        card.push(text_prop(VcardPropKind::Uid, vec![], id));
    }

    // FN is mandatory: the display name, or composed from the parts.
    card.push(text_prop(VcardPropKind::Fn, vec![], &display_name(contact)));

    // NOTE: the Graph title is the honorific (Dr., Mrs.), hence the N
    // prefix slot; the job title maps to TITLE below.
    let n = VcardN {
        family: component(&contact.surname),
        given: component(&contact.given_name),
        additional: component(&contact.middle_name),
        prefixes: component(&contact.title),
        suffixes: Vec::new(),
    };

    let empty = n.family.is_empty()
        && n.given.is_empty()
        && n.additional.is_empty()
        && n.prefixes.is_empty();
    if !empty {
        card.push(VcardProp {
            name: VcardPropName::Kind(VcardPropKind::N),
            params: vec![],
            value: VcardValue::N(n),
        });
    }

    if let Some(nick) = opt(&contact.nick_name) {
        card.push(VcardProp {
            name: VcardPropName::Kind(VcardPropKind::Nickname),
            params: vec![],
            value: VcardValue::TextList(VcardTextList(vec![Cow::Owned(nick.to_string())])),
        });
    }

    for email in list(&contact.email_addresses) {
        if let Some(address) = email.address.as_deref() {
            let address = address.trim();
            if !address.is_empty() {
                card.push(text_prop(VcardPropKind::Email, vec![], address));
            }
        }
    }

    for im in list(&contact.im_addresses) {
        let im = im.trim();
        if !im.is_empty() {
            card.push(VcardProp {
                name: VcardPropName::Kind(VcardPropKind::Impp),
                params: vec![],
                value: VcardValue::Uri(VcardUri(Cow::Owned(im.to_string()))),
            });
        }
    }

    for phone in list(&contact.business_phones) {
        let phone = phone.trim();
        if !phone.is_empty() {
            card.push(text_prop(
                VcardPropKind::Tel,
                vec![type_param("work")],
                phone,
            ));
        }
    }
    for phone in list(&contact.home_phones) {
        let phone = phone.trim();
        if !phone.is_empty() {
            card.push(text_prop(
                VcardPropKind::Tel,
                vec![type_param("home")],
                phone,
            ));
        }
    }
    if let Some(phone) = opt(&contact.mobile_phone) {
        card.push(text_prop(
            VcardPropKind::Tel,
            vec![type_param("cell")],
            phone,
        ));
    }

    if let Some(prop) = adr_prop(contact.home_address.as_option(), Some("home")) {
        card.push(prop);
    }
    if let Some(prop) = adr_prop(contact.business_address.as_option(), Some("work")) {
        card.push(prop);
    }
    if let Some(prop) = adr_prop(contact.other_address.as_option(), None) {
        card.push(prop);
    }

    let company = opt(&contact.company_name);
    let department = opt(&contact.department);
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

    if let Some(title) = opt(&contact.job_title) {
        card.push(text_prop(VcardPropKind::Title, vec![], title));
    }
    if let Some(role) = opt(&contact.profession) {
        card.push(text_prop(VcardPropKind::Role, vec![], role));
    }
    if let Some(page) = opt(&contact.business_home_page) {
        card.push(VcardProp {
            name: VcardPropName::Kind(VcardPropKind::Url),
            params: vec![],
            value: VcardValue::Uri(VcardUri(Cow::Owned(page.to_string()))),
        });
    }

    // NOTE: Graph serves the birthday as an ISO 8601 date-time
    // (1983-04-01T00:00:00Z); only its date part has a vCard slot.
    if let Some(birthday) = opt(&contact.birthday) {
        let date = birthday.split('T').next().unwrap_or(birthday);
        if let Some(date) = full_date(date) {
            card.push(VcardProp {
                name: VcardPropName::Kind(VcardPropKind::Bday),
                params: vec![],
                value: VcardValue::DateAndOrTime(VcardDateAndOrTime(Cow::Owned(date))),
            });
        }
    }

    if let Some(notes) = opt(&contact.personal_notes) {
        card.push(text_prop(VcardPropKind::Note, vec![], notes));
    }

    let categories: Vec<Cow<'static, str>> = list(&contact.categories)
        .iter()
        .map(|category| category.trim())
        .filter(|category| !category.is_empty())
        .map(|category| Cow::Owned(category.to_string()))
        .collect();
    if !categories.is_empty() {
        card.push(VcardProp {
            name: VcardPropName::Kind(VcardPropKind::Categories),
            params: vec![],
            value: VcardValue::TextList(VcardTextList(categories)),
        });
    }

    if let Some(spouse) = opt(&contact.spouse_name) {
        card.push(related_prop("spouse", spouse));
    }
    for child in list(&contact.children) {
        let child = child.trim();
        if !child.is_empty() {
            card.push(related_prop("child", child));
        }
    }

    let vcard = String::from_utf8_lossy(&card.to_bytes()).into_owned();

    // NOTE: Graph-only fields ride the vCard as read-only X-MSGRAPH-*
    // vendor properties, and the stashed remainder restores verbatim
    // (docs/custom-data.md).
    let mut extra = Vec::new();
    if let Some(value) = opt(&contact.file_as) {
        extra.push(format!("X-MSGRAPH-FILE-AS:{}", escape_text(value)));
    }
    if let Some(value) = opt(&contact.office_location) {
        extra.push(format!("X-MSGRAPH-OFFICE-LOCATION:{}", escape_text(value)));
    }
    if let Some(value) = opt(&contact.assistant_name) {
        extra.push(format!("X-MSGRAPH-ASSISTANT:{}", escape_text(value)));
    }
    if let Some(value) = opt(&contact.manager) {
        extra.push(format!("X-MSGRAPH-MANAGER:{}", escape_text(value)));
    }
    extra.extend(stash_lines(contact));

    splice_props(vcard, &extra)
}

/// Projects a vCard onto an io-msgraph contact, the full-state
/// projection: every managed field is Set from the vCard or Null
/// (collections: Set empty) when absent, while unmanaged Graph fields
/// stay Unset. Wire bodies refine it through [`to_new_contact`]
/// (create) and [`to_contact_delta`] (update).
///
/// Graph stores fixed slots, even behind its collection-typed
/// properties: one mobile phone, one home, business and other address,
/// three email addresses, three IM addresses, two business and two
/// home phones. The first matching vCard property fills a slot and the
/// extras are dropped (never misfiled into another slot). The UID is
/// not read back: the Graph id addresses the resource through the
/// request path.
pub fn to_contact(vcard: &str) -> Result<MsgraphContact, String> {
    let card = VcardCst::parse(vcard).map_err(|err| format!("Invalid vCard: {err}"))?;
    let version = card.version();

    let mut display_name = None;
    let mut surname = None;
    let mut given_name = None;
    let mut middle_name = None;
    let mut title = None;
    let mut nick_name = None;
    let mut emails = Vec::new();
    let mut ims = Vec::new();
    let mut business_phones = Vec::new();
    let mut home_phones = Vec::new();
    let mut mobile_phone = None;
    let mut home_address = None;
    let mut business_address = None;
    let mut other_address = None;
    let mut company_name = None;
    let mut department = None;
    let mut job_title = None;
    let mut profession = None;
    let mut business_home_page = None;
    let mut birthday = None;
    let mut spouse_name = None;
    let mut children = Vec::new();
    let mut categories = Vec::new();
    let mut notes = Vec::new();
    let mut stash = Vec::new();
    let mut name_seen = false;

    // NOTE: Outlook stores fixed slots behind the Graph collections:
    // three email addresses (primary/secondary/tertiary), two business
    // and two home phones (Business/Business 2, Home/Home 2), three IM
    // addresses (Graph rejects bodies overflowing a slot set). The
    // first properties win; extras land in the stash remainder like
    // every other line that does not project, so they survive on the
    // server and restore on read (docs/custom-data.md).
    for line in &card.props {
        let consumed = match VcardPropKind::from_str(line.name.get()) {
            // NOTE: the VERSION line is structural and the minted
            // X-MSGRAPH-* properties are read-only projections;
            // neither belongs to the remainder.
            Err(_) => {
                let raw_name = line.name.get();
                raw_name.eq_ignore_ascii_case("VERSION")
                    || MINTED_PROPS
                        .iter()
                        .any(|prop| raw_name.eq_ignore_ascii_case(prop))
            }
            // NOTE: the UID is managed: the Graph id addresses the
            // resource through the request path.
            Ok(VcardPropKind::Uid) => true,
            Ok(VcardPropKind::Fn) => {
                let name = FN::decode(line, version);
                set_first(&mut display_name, &name.0)
            }
            Ok(VcardPropKind::N) => {
                if !name_seen {
                    name_seen = true;
                    let n = N::decode(line, version);
                    set_first(&mut surname, n.family.join(" "));
                    set_first(&mut given_name, n.given.join(" "));
                    set_first(&mut middle_name, n.additional.join(" "));
                    set_first(&mut title, n.prefixes.join(" "));
                    true
                } else {
                    false
                }
            }
            Ok(VcardPropKind::Nickname) => {
                let all = NICKNAME::decode(line, version);
                match all.0.first() {
                    Some(nick) => set_first(&mut nick_name, nick),
                    None => false,
                }
            }
            Ok(VcardPropKind::Email) => {
                let email = EMAIL::decode(line, version);
                let address = email.0.trim();
                if emails.len() < 3 && !address.is_empty() {
                    emails.push(MsgraphEmailAddress {
                        name: None,
                        address: Some(address.to_string()),
                    });
                    true
                } else {
                    false
                }
            }
            Ok(VcardPropKind::Impp) => {
                let impp = IMPP::decode(line, version);
                let address = impp.0.trim();
                if ims.len() < 3 && !address.is_empty() {
                    ims.push(address.to_string());
                    true
                } else {
                    false
                }
            }
            Ok(VcardPropKind::Tel) => {
                let tel = TEL::decode(line, version);
                let number = tel.0.trim();
                if number.is_empty() {
                    false
                } else {
                    let types = type_values(line);
                    if types.iter().any(|t| t == "cell") {
                        // NOTE: one mobile slot; further cell TELs are
                        // stashed rather than misfiled as business
                        // phones.
                        if mobile_phone.is_none() {
                            mobile_phone = Some(number.to_string());
                            true
                        } else {
                            false
                        }
                    } else if types.iter().any(|t| t == "home") {
                        if home_phones.len() < 2 {
                            home_phones.push(number.to_string());
                            true
                        } else {
                            false
                        }
                    } else if business_phones.len() < 2 {
                        business_phones.push(number.to_string());
                        true
                    } else {
                        false
                    }
                }
            }
            Ok(VcardPropKind::Adr) => {
                let adr = ADR::decode(line, version);

                // NOTE: Graph's street is one multiline field; the po
                // box and extended components fold into it so nothing
                // is lost.
                let street: Vec<&str> = adr
                    .po_box
                    .iter()
                    .chain(&adr.extended)
                    .chain(&adr.street)
                    .map(|component| component.as_ref().trim())
                    .filter(|component| !component.is_empty())
                    .collect();

                let address = MsgraphPhysicalAddress {
                    street: (!street.is_empty()).then(|| street.join("\n")),
                    city: joined(&adr.locality),
                    state: joined(&adr.region),
                    country_or_region: joined(&adr.country),
                    postal_code: joined(&adr.postal_code),
                };

                let empty = address.street.is_none()
                    && address.city.is_none()
                    && address.state.is_none()
                    && address.country_or_region.is_none()
                    && address.postal_code.is_none();
                if empty {
                    false
                } else {
                    let types = type_values(line);
                    let slot = if types.iter().any(|t| t == "home") {
                        &mut home_address
                    } else if types.iter().any(|t| t == "work") {
                        &mut business_address
                    } else {
                        &mut other_address
                    };
                    if slot.is_none() {
                        *slot = Some(address);
                        true
                    } else {
                        false
                    }
                }
            }
            Ok(VcardPropKind::Org) => {
                if company_name.is_none() && department.is_none() {
                    let org = ORG::decode(line, version);
                    let mut components = org.0.iter().map(|component| component.as_ref());
                    let company = components.next().unwrap_or_default();
                    let rest = components.collect::<Vec<_>>().join(" ");
                    set_first(&mut company_name, company);
                    set_first(&mut department, rest);
                    true
                } else {
                    false
                }
            }
            Ok(VcardPropKind::Title) => {
                let value = TITLE::decode(line, version);
                set_first(&mut job_title, &value.0)
            }
            Ok(VcardPropKind::Role) => {
                let role = ROLE::decode(line, version);
                set_first(&mut profession, &role.0)
            }
            Ok(VcardPropKind::Url) => {
                let url = URL::decode(line, version);
                set_first(&mut business_home_page, &url.0)
            }
            Ok(VcardPropKind::Bday) => {
                // NOTE: a partial (year-less) birthday has no Graph
                // date it round-trips through; it lands in the stash.
                if birthday.is_none()
                    && let Some(date) = full_date(&line.raw_value_str())
                {
                    birthday = Some(format!("{date}T00:00:00Z"));
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
            Ok(VcardPropKind::Categories) => {
                let all = CATEGORIES::decode(line, version);
                let mut pushed = false;
                for category in &all.0 {
                    let category = category.trim();
                    if !category.is_empty() {
                        categories.push(category.to_string());
                        pushed = true;
                    }
                }
                pushed
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
                if text {
                    let related = RELATED::decode(line, version);
                    let types = type_values(line);
                    if types.iter().any(|t| t == "spouse") {
                        set_first(&mut spouse_name, &related.0)
                    } else if types.iter().any(|t| t == "child") {
                        let child = related.0.trim();
                        if !child.is_empty() {
                            children.push(child.to_string());
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
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

    Ok(MsgraphContact {
        display_name: MsgraphField::set_or_null(display_name),
        given_name: MsgraphField::set_or_null(given_name),
        middle_name: MsgraphField::set_or_null(middle_name),
        surname: MsgraphField::set_or_null(surname),
        nick_name: MsgraphField::set_or_null(nick_name),
        title: MsgraphField::set_or_null(title),
        email_addresses: MsgraphField::Set(emails),
        im_addresses: MsgraphField::Set(ims),
        business_phones: MsgraphField::Set(business_phones),
        home_phones: MsgraphField::Set(home_phones),
        mobile_phone: MsgraphField::set_or_null(mobile_phone),
        home_address: MsgraphField::set_or_null(home_address),
        business_address: MsgraphField::set_or_null(business_address),
        other_address: MsgraphField::set_or_null(other_address),
        job_title: MsgraphField::set_or_null(job_title),
        company_name: MsgraphField::set_or_null(company_name),
        department: MsgraphField::set_or_null(department),
        profession: MsgraphField::set_or_null(profession),
        business_home_page: MsgraphField::set_or_null(business_home_page),
        birthday: MsgraphField::set_or_null(birthday),
        spouse_name: MsgraphField::set_or_null(spouse_name),
        children: MsgraphField::Set(children),
        personal_notes: MsgraphField::set_or_null((!notes.is_empty()).then(|| notes.join("\n"))),
        categories: MsgraphField::Set(categories),
        // NOTE: the stash entry is always Set (empty value when the
        // card has no remainder) so the delta projection can tell a
        // cleared stash from an unchanged one.
        single_value_extended_properties: MsgraphField::Set(vec![
            MsgraphSingleValueExtendedProperty {
                id: EXTENDED_PROP_ID.to_string(),
                value: stash.join("\n"),
            },
        ]),
        ..Default::default()
    })
}

/// Projects a vCard onto an io-msgraph contact for an update body,
/// reduced to the fields that differ from the base vCard (the state
/// last synced with the server): unchanged fields turn Unset and stay
/// out of the PATCH. The full-state body nulls every absent field and
/// the Outlook backend rejects such gratuitous nulls with an internal
/// server error (HTTP 500); here a Null only survives when the edit
/// actually cleared a field the base carried (collections clear
/// through an explicit empty Set instead).
pub fn to_contact_delta(vcard: &str, base_vcard: &str) -> Result<MsgraphContact, String> {
    let mut contact = to_contact(vcard)?;
    let base = to_contact(base_vcard)?;

    macro_rules! unset_unchanged {
        ($($field:ident),* $(,)?) => {$(
            if contact.$field == base.$field {
                contact.$field = MsgraphField::Unset;
            }
        )*};
    }

    unset_unchanged!(
        display_name,
        given_name,
        middle_name,
        surname,
        nick_name,
        title,
        email_addresses,
        im_addresses,
        business_phones,
        home_phones,
        mobile_phone,
        home_address,
        business_address,
        other_address,
        job_title,
        company_name,
        department,
        profession,
        business_home_page,
        birthday,
        spouse_name,
        children,
        personal_notes,
        categories,
        single_value_extended_properties,
    );

    Ok(contact)
}

/// Projects a vCard onto an io-msgraph contact for a create body: the
/// full-state projection minus the explicit nulls and empty
/// collections. They are no-ops on a fresh resource, and the Outlook
/// backend rejects null complex properties on POST with an internal
/// server error (HTTP 500).
pub fn to_new_contact(vcard: &str) -> Result<MsgraphContact, String> {
    let mut contact = to_contact(vcard)?;

    // NOTE: an empty stash entry is a no-op on a fresh resource.
    if stash_lines(&contact).is_empty() {
        contact.single_value_extended_properties = MsgraphField::Unset;
    }

    let mut body = to_value(&contact).map_err(|err| err.to_string())?;
    if let Some(object) = body.as_object_mut() {
        object.retain(|_, value| {
            !value.is_null() && value.as_array().is_none_or(|array| !array.is_empty())
        });
    }

    from_value(body).map_err(|err| err.to_string())
}

/// The contact's display name, composed from the name parts when
/// blank, with the filing name as a last resort.
fn display_name(contact: &MsgraphContact) -> String {
    if let Some(name) = opt(&contact.display_name) {
        return name.to_string();
    }

    let composed: Vec<&str> = [&contact.given_name, &contact.middle_name, &contact.surname]
        .into_iter()
        .filter_map(opt)
        .collect();
    if !composed.is_empty() {
        return composed.join(" ");
    }

    opt(&contact.file_as).unwrap_or_default().to_string()
}

/// An ADR property from a Graph physical address, or None when every
/// component is empty. Graph's street is one multiline field, so each
/// of its lines becomes a vCard street component.
fn adr_prop(
    address: Option<&MsgraphPhysicalAddress>,
    r#type: Option<&'static str>,
) -> Option<VcardProp<'static>> {
    let address = address?;

    let street = address.street.as_deref().unwrap_or("");
    let value = VcardAdr {
        po_box: Vec::new(),
        extended: Vec::new(),
        street: street
            .split('\n')
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(|line| Cow::Owned(line.to_string()))
            .collect(),
        locality: option_component(&address.city),
        region: option_component(&address.state),
        postal_code: option_component(&address.postal_code),
        country: option_component(&address.country_or_region),
        ..Default::default()
    };

    let empty = value.street.is_empty()
        && value.locality.is_empty()
        && value.region.is_empty()
        && value.postal_code.is_empty()
        && value.country.is_empty();
    if empty {
        return None;
    }

    Some(VcardProp {
        name: VcardPropName::Kind(VcardPropKind::Adr),
        params: r#type.map(type_param).into_iter().collect(),
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

/// The trimmed field, None when unset, null or blank.
fn opt(field: &MsgraphField<String>) -> Option<&str> {
    let value = field.as_deref()?.trim();
    if value.is_empty() { None } else { Some(value) }
}

/// The collection items, empty when the field is unset or null.
fn list<T>(field: &MsgraphField<Vec<T>>) -> &[T] {
    field.as_option().map(Vec::as_slice).unwrap_or(&[])
}

/// A structured-value component holding the trimmed field, or empty.
fn component(field: &MsgraphField<String>) -> Vec<Cow<'static, str>> {
    match opt(field) {
        Some(value) => vec![Cow::Owned(value.to_string())],
        None => Vec::new(),
    }
}

/// Like [`component`], for the plain options of a physical address.
fn option_component(value: &Option<String>) -> Vec<Cow<'static, str>> {
    let value = value.as_deref().unwrap_or("").trim();
    if value.is_empty() {
        Vec::new()
    } else {
        vec![Cow::Owned(value.to_string())]
    }
}

/// Fills a single-instance Graph field, first non-empty value wins;
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

/// The stashed vCard remainder lines behind the contact's cardamum
/// extended property (matched by name: Graph may normalize the GUID
/// spelling in responses).
fn stash_lines(contact: &MsgraphContact) -> Vec<String> {
    contact
        .single_value_extended_properties
        .as_option()
        .map(Vec::as_slice)
        .unwrap_or(&[])
        .iter()
        .filter(|prop| prop.id.to_ascii_lowercase().contains("cardamum-vcard"))
        .flat_map(|prop| prop.value.split('\n'))
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

/// The structured-value components joined into one Graph field, None
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

    /// A contact filling every field the projection manages.
    fn full_contact() -> MsgraphContact {
        MsgraphContact {
            display_name: MsgraphField::Set("Jane Doe".into()),
            given_name: MsgraphField::Set("Jane".into()),
            middle_name: MsgraphField::Set("Q.".into()),
            surname: MsgraphField::Set("Doe".into()),
            title: MsgraphField::Set("Dr.".into()),
            nick_name: MsgraphField::Set("Janie".into()),
            email_addresses: MsgraphField::Set(vec![MsgraphEmailAddress {
                name: None,
                address: Some("jane@doe.org".into()),
            }]),
            im_addresses: MsgraphField::Set(vec!["xmpp:jane@doe.org".into()]),
            business_phones: MsgraphField::Set(vec!["+331111".into()]),
            home_phones: MsgraphField::Set(vec!["+332222".into()]),
            mobile_phone: MsgraphField::Set("+333333".into()),
            home_address: MsgraphField::Set(MsgraphPhysicalAddress {
                street: Some("12 Main St".into()),
                city: Some("Paris".into()),
                state: Some("IDF".into()),
                country_or_region: Some("France".into()),
                postal_code: Some("75000".into()),
            }),
            business_address: MsgraphField::Set(MsgraphPhysicalAddress {
                street: Some("1 Work Way".into()),
                city: Some("Lyon".into()),
                state: None,
                country_or_region: None,
                postal_code: None,
            }),
            other_address: MsgraphField::Set(MsgraphPhysicalAddress {
                street: Some("3 Other Rd".into()),
                city: None,
                state: None,
                country_or_region: None,
                postal_code: None,
            }),
            company_name: MsgraphField::Set("ACME".into()),
            department: MsgraphField::Set("R&D".into()),
            job_title: MsgraphField::Set("Boss".into()),
            profession: MsgraphField::Set("Engineer".into()),
            business_home_page: MsgraphField::Set("https://doe.org".into()),
            birthday: MsgraphField::Set("1983-04-01T00:00:00Z".into()),
            spouse_name: MsgraphField::Set("John Doe".into()),
            children: MsgraphField::Set(vec!["Jimmy".into()]),
            personal_notes: MsgraphField::Set("a note".into()),
            categories: MsgraphField::Set(vec!["friends".into(), "band".into()]),
            ..Default::default()
        }
    }

    #[test]
    fn to_vcard_projects_every_mapped_field() {
        let mut contact = full_contact();
        contact.id = "AAMkAGI2".into();

        let vcard = to_vcard(&contact);
        assert!(vcard.contains("VERSION:4.0\r\n"));
        assert!(vcard.contains("UID:AAMkAGI2\r\n"));
        assert!(vcard.contains("FN:Jane Doe\r\n"));
        assert!(vcard.contains("N:Doe;Jane;Q.;Dr.;\r\n"));
        assert!(vcard.contains("NICKNAME:Janie\r\n"));
        assert!(vcard.contains("EMAIL:jane@doe.org\r\n"));
        assert!(vcard.contains("IMPP:xmpp:jane@doe.org\r\n"));
        assert!(vcard.contains("TEL;TYPE=work:+331111\r\n"));
        assert!(vcard.contains("TEL;TYPE=home:+332222\r\n"));
        assert!(vcard.contains("TEL;TYPE=cell:+333333\r\n"));
        assert!(vcard.contains("ADR;TYPE=home:;;12 Main St;Paris;IDF;75000;France\r\n"));
        assert!(vcard.contains("ADR;TYPE=work:;;1 Work Way;Lyon;;;\r\n"));
        assert!(vcard.contains("ADR:;;3 Other Rd;;;;\r\n"));
        assert!(vcard.contains("ORG:ACME;R&D\r\n"));
        assert!(vcard.contains("TITLE:Boss\r\n"));
        assert!(vcard.contains("ROLE:Engineer\r\n"));
        assert!(vcard.contains("URL:https://doe.org\r\n"));
        assert!(vcard.contains("BDAY:1983-04-01\r\n"));
        assert!(vcard.contains("NOTE:a note\r\n"));
        assert!(vcard.contains("CATEGORIES:friends,band\r\n"));
        assert!(vcard.contains("RELATED;TYPE=spouse;VALUE=text:John Doe\r\n"));
        assert!(vcard.contains("RELATED;TYPE=child;VALUE=text:Jimmy\r\n"));
    }

    #[test]
    fn to_vcard_composes_fn_from_name_parts() {
        let contact = MsgraphContact {
            given_name: MsgraphField::Set("Jane".into()),
            surname: MsgraphField::Set("Doe".into()),
            ..Default::default()
        };
        assert!(to_vcard(&contact).contains("FN:Jane Doe\r\n"));
    }

    #[test]
    fn to_contact_reads_managed_props() {
        let vcard = "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:abc\r\nFN:Jane Doe\r\n\
            N:Doe;Jane;Q.;Dr.;\r\nNICKNAME:Janie,JJ\r\nEMAIL:jane@doe.org\r\n\
            IMPP:xmpp:jane@doe.org\r\nTEL;TYPE=cell:+333333\r\n\
            TEL;TYPE=home,voice:+332222\r\nTEL:+331111\r\n\
            ADR;TYPE=home:;;12 Main St;Paris;IDF;75000;France\r\nORG:ACME;R&D\r\n\
            TITLE:Boss\r\nROLE:Engineer\r\nURL:https://doe.org\r\nBDAY:1983-04-01\r\n\
            NOTE:a note\r\nCATEGORIES:friends,band\r\n\
            RELATED;TYPE=spouse;VALUE=text:John Doe\r\n\
            RELATED;TYPE=child;VALUE=text:Jimmy\r\nEND:VCARD\r\n";

        let contact = to_contact(vcard).unwrap();
        assert_eq!(contact.id, "");
        assert_eq!(contact.display_name.as_deref(), Some("Jane Doe"));
        assert_eq!(contact.surname.as_deref(), Some("Doe"));
        assert_eq!(contact.given_name.as_deref(), Some("Jane"));
        assert_eq!(contact.middle_name.as_deref(), Some("Q."));
        assert_eq!(contact.title.as_deref(), Some("Dr."));
        assert_eq!(contact.nick_name.as_deref(), Some("Janie"));
        let emails = contact.email_addresses.as_option().unwrap();
        assert_eq!(emails[0].address.as_deref(), Some("jane@doe.org"));
        assert_eq!(
            contact.im_addresses.as_option().unwrap(),
            &vec!["xmpp:jane@doe.org".to_string()]
        );
        assert_eq!(contact.mobile_phone.as_deref(), Some("+333333"));
        assert_eq!(
            contact.home_phones.as_option().unwrap(),
            &vec!["+332222".to_string()]
        );
        assert_eq!(
            contact.business_phones.as_option().unwrap(),
            &vec!["+331111".to_string()]
        );
        let home = contact.home_address.as_option().unwrap();
        assert_eq!(home.street.as_deref(), Some("12 Main St"));
        assert_eq!(home.city.as_deref(), Some("Paris"));
        assert_eq!(home.state.as_deref(), Some("IDF"));
        assert_eq!(home.postal_code.as_deref(), Some("75000"));
        assert_eq!(home.country_or_region.as_deref(), Some("France"));
        assert_eq!(contact.company_name.as_deref(), Some("ACME"));
        assert_eq!(contact.department.as_deref(), Some("R&D"));
        assert_eq!(contact.job_title.as_deref(), Some("Boss"));
        assert_eq!(contact.profession.as_deref(), Some("Engineer"));
        assert_eq!(
            contact.business_home_page.as_deref(),
            Some("https://doe.org")
        );
        assert_eq!(contact.birthday.as_deref(), Some("1983-04-01T00:00:00Z"));
        assert_eq!(contact.spouse_name.as_deref(), Some("John Doe"));
        assert_eq!(
            contact.children.as_option().unwrap(),
            &vec!["Jimmy".to_string()]
        );
        assert_eq!(contact.personal_notes.as_deref(), Some("a note"));
        assert_eq!(
            contact.categories.as_option().unwrap(),
            &vec!["friends".to_string(), "band".to_string()]
        );
    }

    #[test]
    fn to_contact_clears_absent_managed_fields_and_keeps_unmanaged_unset() {
        let vcard = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:X\r\nEND:VCARD\r\n";
        let contact = to_contact(vcard).unwrap();

        // Managed but absent from the vCard: cleared by the PATCH.
        assert_eq!(contact.spouse_name, MsgraphField::Null);
        assert_eq!(contact.mobile_phone, MsgraphField::Null);
        assert_eq!(contact.home_address, MsgraphField::Null);
        assert_eq!(contact.business_phones, MsgraphField::Set(vec![]));
        assert_eq!(contact.email_addresses, MsgraphField::Set(vec![]));

        // Unmanaged Graph fields: out of the body, preserved.
        assert_eq!(contact.file_as, MsgraphField::Unset);
        assert_eq!(contact.assistant_name, MsgraphField::Unset);
        assert_eq!(contact.manager, MsgraphField::Unset);
        assert_eq!(contact.office_location, MsgraphField::Unset);
    }

    #[test]
    fn to_contact_delta_keeps_only_changed_fields() {
        let base = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane Doe\r\n\
            TEL;TYPE=cell:+333333\r\nEMAIL:jane@doe.org\r\n\
            NOTE:a note\r\nEND:VCARD\r\n";
        let edited = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane Doe\r\n\
            TEL;TYPE=cell:+444444\r\nEMAIL:jane@doe.org\r\nEND:VCARD\r\n";

        let contact = to_contact_delta(edited, base).unwrap();

        // Changed: the new value; cleared: an explicit Null.
        assert_eq!(contact.mobile_phone.as_deref(), Some("+444444"));
        assert_eq!(contact.personal_notes, MsgraphField::Null);

        // Unchanged (set or absent on both sides): out of the body,
        // no gratuitous nulls for the Outlook backend to choke on.
        assert_eq!(contact.display_name, MsgraphField::Unset);
        assert_eq!(contact.email_addresses, MsgraphField::Unset);
        assert_eq!(contact.birthday, MsgraphField::Unset);
        assert_eq!(contact.home_address, MsgraphField::Unset);
        assert_eq!(contact.categories, MsgraphField::Unset);
    }

    #[test]
    fn to_contact_delta_clears_collections_with_empty_sets() {
        let base = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:X\r\n\
            TEL;TYPE=home:+332222\r\nCATEGORIES:friends\r\nEND:VCARD\r\n";
        let edited = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:X\r\nEND:VCARD\r\n";

        let contact = to_contact_delta(edited, base).unwrap();
        assert_eq!(contact.home_phones, MsgraphField::Set(vec![]));
        assert_eq!(contact.categories, MsgraphField::Set(vec![]));
    }

    #[test]
    fn to_contact_delta_of_identical_cards_is_empty() {
        let vcard = to_vcard(&full_contact());
        let contact = to_contact_delta(&vcard, &vcard).unwrap();
        assert_eq!(contact, MsgraphContact::default());
    }

    #[test]
    fn to_new_contact_drops_nulls_and_empty_collections() {
        let vcard = "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:X\r\nTEL;TYPE=CELL:+331111\r\nEND:VCARD\r\n";
        let contact = to_new_contact(vcard).unwrap();

        assert_eq!(contact.display_name.as_deref(), Some("X"));
        assert_eq!(contact.mobile_phone.as_deref(), Some("+331111"));
        assert_eq!(contact.spouse_name, MsgraphField::Unset);
        assert_eq!(contact.home_address, MsgraphField::Unset);
        assert_eq!(contact.business_phones, MsgraphField::Unset);
        assert_eq!(contact.email_addresses, MsgraphField::Unset);
    }

    #[test]
    fn to_contact_respects_graph_slots() {
        let vcard = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:X\r\n\
            TEL;TYPE=cell:+331111\r\nTEL;TYPE=cell:+332222\r\n\
            TEL;TYPE=work:+333331\r\nTEL;TYPE=work:+333332\r\nTEL:+333333\r\n\
            TEL;TYPE=home:+334441\r\nTEL;TYPE=home:+334442\r\nTEL;TYPE=home:+334443\r\n\
            ADR;TYPE=home:;;First St;;;;\r\nADR;TYPE=home:;;Second St;;;;\r\n\
            EMAIL:a@x.org\r\nEMAIL:b@x.org\r\nEMAIL:c@x.org\r\nEMAIL:d@x.org\r\n\
            IMPP:xmpp:a@x.org\r\nIMPP:xmpp:b@x.org\r\nIMPP:xmpp:c@x.org\r\nIMPP:xmpp:d@x.org\r\n\
            RELATED;TYPE=spouse:urn:uuid:03a0e51f\r\nEND:VCARD\r\n";

        let contact = to_contact(vcard).unwrap();

        // One mobile slot: the second cell TEL is dropped, not
        // misfiled as a business phone.
        assert_eq!(contact.mobile_phone.as_deref(), Some("+331111"));

        // Two business, two home phone slots; three email and IM slots.
        assert_eq!(
            contact.business_phones.as_option().unwrap(),
            &vec!["+333331".to_string(), "+333332".to_string()]
        );
        assert_eq!(
            contact.home_phones.as_option().unwrap(),
            &vec!["+334441".to_string(), "+334442".to_string()]
        );
        assert_eq!(contact.email_addresses.as_option().unwrap().len(), 3);
        assert_eq!(contact.im_addresses.as_option().unwrap().len(), 3);

        // One address per home, work and other slot.
        let home = contact.home_address.as_option().unwrap();
        assert_eq!(home.street.as_deref(), Some("First St"));
        assert_eq!(contact.spouse_name, MsgraphField::Null);
    }

    #[test]
    fn round_trip() {
        let contact = full_contact();

        // NOTE: the full-state projection always carries the (empty)
        // stash slot, so a cleared stash is distinguishable from an
        // unchanged one.
        let mut expected = contact.clone();
        expected.single_value_extended_properties =
            MsgraphField::Set(vec![MsgraphSingleValueExtendedProperty {
                id: EXTENDED_PROP_ID.to_string(),
                value: String::new(),
            }]);

        assert_eq!(to_contact(&to_vcard(&contact)).unwrap(), expected);
    }

    #[test]
    fn stash_preserves_unprojected_props() {
        let vcard = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:X\r\nX-FOO;TYPE=bar:baz\r\n\
            EMAIL:a@x.org\r\nEMAIL:b@x.org\r\nEMAIL:c@x.org\r\nEMAIL:d@x.org\r\nEND:VCARD\r\n";

        let contact = to_contact(vcard).unwrap();
        let props = contact
            .single_value_extended_properties
            .as_option()
            .unwrap();
        assert_eq!(props[0].id, EXTENDED_PROP_ID);
        assert_eq!(props[0].value, "X-FOO;TYPE=bar:baz\nEMAIL:d@x.org");

        let restored = to_vcard(&contact);
        assert!(restored.contains("X-FOO;TYPE=bar:baz\r\n"));
        assert!(restored.contains("EMAIL:d@x.org\r\n"));
        assert!(restored.ends_with("END:VCARD\r\n"));
    }

    #[test]
    fn minted_props_project_and_consume() {
        let mut contact = full_contact();
        contact.file_as = MsgraphField::Set("Doe, Jane".into());
        contact.office_location = MsgraphField::Set("B2".into());
        contact.assistant_name = MsgraphField::Set("Sam".into());
        contact.manager = MsgraphField::Set("Alex".into());

        let vcard = to_vcard(&contact);
        assert!(vcard.contains("X-MSGRAPH-FILE-AS:Doe\\, Jane\r\n"));
        assert!(vcard.contains("X-MSGRAPH-OFFICE-LOCATION:B2\r\n"));
        assert!(vcard.contains("X-MSGRAPH-ASSISTANT:Sam\r\n"));
        assert!(vcard.contains("X-MSGRAPH-MANAGER:Alex\r\n"));

        // Consumed on the way back: the fields stay Unset (out of the
        // body, server authoritative) and the lines stay out of the
        // stash.
        let back = to_contact(&vcard).unwrap();
        assert_eq!(back.file_as, MsgraphField::Unset);
        assert_eq!(back.office_location, MsgraphField::Unset);
        assert_eq!(back.assistant_name, MsgraphField::Unset);
        assert_eq!(back.manager, MsgraphField::Unset);
        let props = back.single_value_extended_properties.as_option().unwrap();
        assert_eq!(props[0].value, "");
    }

    #[test]
    fn to_contact_delta_tracks_the_stash() {
        let base = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:X\r\nX-FOO:bar\r\nEND:VCARD\r\n";
        let unchanged = to_contact_delta(base, base).unwrap();
        assert_eq!(
            unchanged.single_value_extended_properties,
            MsgraphField::Unset
        );

        // A dropped remainder clears the server-side stash through an
        // explicit empty value.
        let cleared = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:X\r\nEND:VCARD\r\n";
        let delta = to_contact_delta(cleared, base).unwrap();
        let props = delta.single_value_extended_properties.as_option().unwrap();
        assert_eq!(props[0].value, "");
    }

    #[test]
    fn to_new_contact_drops_the_empty_stash() {
        let vcard = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:X\r\nEND:VCARD\r\n";
        let contact = to_new_contact(vcard).unwrap();
        assert_eq!(
            contact.single_value_extended_properties,
            MsgraphField::Unset
        );

        let vcard = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:X\r\nX-FOO:bar\r\nEND:VCARD\r\n";
        let contact = to_new_contact(vcard).unwrap();
        let props = contact
            .single_value_extended_properties
            .as_option()
            .unwrap();
        assert_eq!(props[0].value, "X-FOO:bar");
    }
}
