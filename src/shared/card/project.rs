//! # Card projection
//!
//! The common fields of a vCard, read through vcard-rs's decoded model
//! rather than scanned off its lines: a folded line, an escaped comma
//! and a `tel:` URI come back as the values they stand for.
//!
//! A projection is read-only and lossy by design. The vCard stays the
//! record, `card read` printing it whole.

use schemars::JsonSchema;
use serde::Serialize;
use vcard::{
    prop::{VcardProp, VcardPropKind, VcardPropName},
    tree::{cst::VcardCst, line::VcardLine, value::cursor::VcardValueCursor},
    value::VcardValue,
};

/// The common fields of a card, as listings and reads print them.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CardFields {
    /// The `UID`.
    pub uid: Option<String>,
    /// The display name (`FN`).
    pub full_name: Option<String>,
    /// The given names of `N`, space-joined.
    pub given_name: Option<String>,
    /// The family names of `N`, space-joined.
    pub family_name: Option<String>,
    /// Every `EMAIL`, in document order.
    pub emails: Vec<String>,
    /// Every `TEL`, in document order, a `tel:` URI as its number.
    pub phones: Vec<String>,
    /// The organization name, the first component of `ORG`.
    pub organization: Option<String>,
    /// The organizational units, the other components of `ORG`.
    pub organization_units: Vec<String>,
    /// The job title (`TITLE`).
    pub title: Option<String>,
    /// The free-form note (`NOTE`).
    pub note: Option<String>,
}

impl CardFields {
    /// Projects the first card of `contents`; one that does not parse, or
    /// is not UTF-8, projects to nothing rather than failing the listing.
    pub fn project(contents: &[u8]) -> Self {
        // NOTE: the parser reads bytes that are not UTF-8 lossily, and a
        // field never carries a lossy decode.
        if std::str::from_utf8(contents).is_err() {
            return Self::default();
        }

        let Ok(mut cst) = VcardCst::parse(contents) else {
            return Self::default();
        };

        let transcoded: Vec<Option<String>> = cst
            .props
            .iter_mut()
            .filter(|line| !line.name.get().eq_ignore_ascii_case("VERSION"))
            .map(transcode)
            .collect();

        let card = cst.decode();
        let props: Vec<(&VcardProp<'_>, Option<&str>)> = card
            .properties
            .iter()
            .zip(transcoded.iter().map(Option::as_deref))
            .collect();

        let first_text = |kind: VcardPropKind| {
            props
                .iter()
                .filter(|(prop, _)| is(prop, kind))
                .find_map(|(prop, raw)| read_text(prop, *raw))
        };
        let every_text = |kind: VcardPropKind| -> Vec<String> {
            props
                .iter()
                .filter(|(prop, _)| is(prop, kind))
                .filter_map(|(prop, raw)| read_text(prop, *raw))
                .collect()
        };
        let components = |kind: VcardPropKind| -> Option<Vec<Vec<String>>> {
            props
                .iter()
                .find(|(prop, _)| is(prop, kind))
                .and_then(|(prop, raw)| read_components(prop, *raw))
        };

        let join = |parts: Option<&Vec<String>>| {
            let joined = parts?
                .iter()
                .map(|part| part.trim())
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            (!joined.is_empty()).then_some(joined)
        };

        let name = components(VcardPropKind::N).unwrap_or_default();
        let org: Vec<String> = components(VcardPropKind::Org)
            .unwrap_or_default()
            .into_iter()
            .map(|unit| unit.join(",").trim().to_owned())
            .collect();

        Self {
            uid: first_text(VcardPropKind::Uid),
            full_name: first_text(VcardPropKind::Fn),
            given_name: join(name.get(1)),
            family_name: join(name.first()),
            emails: every_text(VcardPropKind::Email),
            phones: every_text(VcardPropKind::Tel)
                .into_iter()
                .map(|phone| match phone.get(..4) {
                    Some(scheme) if scheme.eq_ignore_ascii_case("tel:") => phone[4..].to_owned(),
                    _ => phone,
                })
                .collect(),
            organization: org.first().filter(|name| !name.is_empty()).cloned(),
            organization_units: org
                .into_iter()
                .skip(1)
                .filter(|unit| !unit.is_empty())
                .collect(),
            title: first_text(VcardPropKind::Title),
            note: first_text(VcardPropKind::Note),
        }
    }
}

/// Whether a property is of a kind.
fn is(prop: &VcardProp<'_>, kind: VcardPropKind) -> bool {
    matches!(prop.name, VcardPropName::Kind(found) if found == kind)
}

/// The text of a line declaring `QUOTED-PRINTABLE` or a `CHARSET` (vCard
/// 2.1), decoded and transcoded to UTF-8, its `;` kept literal; none for
/// any other line. The decoded model keeps such a value raw, its cursor
/// alone decodes it.
pub(crate) fn transcode(line: &mut VcardLine<'_>) -> Option<String> {
    let declares = line.params.iter().any(|param| {
        let name = param.name.get();
        let quoted_printable = |value: &str| value.eq_ignore_ascii_case("QUOTED-PRINTABLE");

        name.eq_ignore_ascii_case("CHARSET")
            || (name.eq_ignore_ascii_case("ENCODING")
                && param
                    .values
                    .iter()
                    .any(|value| quoted_printable(value.get())))
            || (param.values.is_empty() && quoted_printable(name))
    });

    declares.then(|| VcardValueCursor { line }.charset())
}

/// A property read as text, from its transcoded text when it has one.
fn read_text(prop: &VcardProp<'_>, transcoded: Option<&str>) -> Option<String> {
    match transcoded {
        Some(raw) => {
            let raw = raw.trim();
            (!raw.is_empty()).then(|| raw.to_owned())
        }
        None => text(&prop.value),
    }
}

/// A structured property (`N`, `ORG`) read as its components, each a list
/// of values, from its transcoded text when it has one.
fn read_components(prop: &VcardProp<'_>, transcoded: Option<&str>) -> Option<Vec<Vec<String>>> {
    let owned = |parts: &[std::borrow::Cow<'_, str>]| {
        parts
            .iter()
            .map(|part| part.to_string())
            .collect::<Vec<_>>()
    };

    match (transcoded, &prop.value) {
        (Some(raw), _) => Some(
            raw.split(';')
                .map(|component| component.split(',').map(str::to_owned).collect())
                .collect(),
        ),
        (None, VcardValue::N(name)) => Some(vec![owned(&name.family), owned(&name.given)]),
        (None, VcardValue::Org(org)) => {
            Some(org.0.iter().map(|unit| vec![unit.to_string()]).collect())
        }
        (None, VcardValue::Text(text)) => Some(vec![vec![text.0.to_string()]]),
        _ => None,
    }
}

/// A value read as text, empty ones counting as none.
fn text(value: &VcardValue<'_>) -> Option<String> {
    let text = match value {
        VcardValue::Text(text) => text.0.trim().to_owned(),
        VcardValue::Uri(uri) => uri.0.trim().to_owned(),
        VcardValue::Unknown(unknown) => unknown
            .components
            .iter()
            .map(|values| values.join(","))
            .collect::<Vec<_>>()
            .join(";"),
        _ => return None,
    };

    (!text.is_empty()).then_some(text)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn the_common_fields_project_with_their_escapes_and_folds_resolved() {
        let card = b"BEGIN:VCARD\r\nVERSION:4.0\r\nUID:urn:uuid:c1\r\n\
            FN:Doe\\, Jane\r\nN:Doe;Jane;Ann;Dr.;\r\n\
            EMAIL;TYPE=work:jane@example.org\r\nEMAIL:jane.doe@example.org\r\n\
            TEL;VALUE=uri;TYPE=cell:tel:+33-6-00-00-00-00\r\nTEL:+33 1 00 00 00 00\r\n\
            ORG:Example Corp;Research\r\nTITLE:Chief\r\n  Scientist\r\n\
            NOTE:Met at the fair\\nlikes tea\r\nEND:VCARD\r\n";

        let fields = serde_json::to_value(CardFields::project(card)).unwrap();

        assert_eq!(
            fields,
            json!({
                "uid": "urn:uuid:c1",
                "fullName": "Doe, Jane",
                "givenName": "Jane",
                "familyName": "Doe",
                "emails": ["jane@example.org", "jane.doe@example.org"],
                "phones": ["+33-6-00-00-00-00", "+33 1 00 00 00 00"],
                "organization": "Example Corp",
                "organizationUnits": ["Research"],
                "title": "Chief Scientist",
                "note": "Met at the fair\nlikes tea",
            })
        );
    }

    #[test]
    fn a_vcard_3_card_projects_alike() {
        let card = b"BEGIN:VCARD\r\nVERSION:3.0\r\nFN:John Roe\r\nN:Roe;John;;;\r\n\
            EMAIL;TYPE=INTERNET,HOME:john@example.org\r\nTEL;TYPE=WORK,VOICE:+1 555 0100\r\n\
            ORG:Roe & Co\r\nEND:VCARD\r\n";

        let fields = CardFields::project(card);

        assert_eq!(fields.full_name.as_deref(), Some("John Roe"));
        assert_eq!(fields.emails, ["john@example.org"]);
        assert_eq!(fields.phones, ["+1 555 0100"]);
        assert_eq!(fields.organization.as_deref(), Some("Roe & Co"));
        assert!(fields.organization_units.is_empty());
        assert_eq!(fields.title, None);
    }

    #[test]
    fn a_vcard_2_1_card_projects_its_quoted_printable_values_decoded() {
        let card = b"BEGIN:VCARD\r\nVERSION:2.1\r\n\
            FN;CHARSET=UTF-8;ENCODING=QUOTED-PRINTABLE:Andr=C3=A9 Dupr=C3=A9\r\n\
            N;CHARSET=UTF-8;ENCODING=QUOTED-PRINTABLE:Dupr=C3=A9;Andr=C3=A9;;;\r\n\
            NOTE;CHARSET=ISO-8859-1;QUOTED-PRINTABLE:Caf=E9\r\nEND:VCARD\r\n";

        let fields = CardFields::project(card);

        assert_eq!(fields.full_name.as_deref(), Some("André Dupré"));
        assert_eq!(fields.given_name.as_deref(), Some("André"));
        assert_eq!(fields.family_name.as_deref(), Some("Dupré"));
        assert_eq!(fields.note.as_deref(), Some("Café"));
    }

    #[test]
    fn an_unparseable_card_projects_nothing() {
        assert_eq!(CardFields::project(b"not a card"), CardFields::default());
    }
}
