//! JMAP ContactCard (RFC 9610) to vCard projection and back, via
//! vcard-rs's JSContact conversion (RFC 9555). The ContactCard's
//! JSContact payload (RFC 9553) converts losslessly: vCard properties
//! with no JSContact counterpart ride the standard `vCardProps` escape
//! hatch both ways, so the vCard document of record round-trips.
//!
//! JMAP has no per-card ETag; the revision surfaced by [`to_card`] is
//! a hash of the card's JSON, which only drives display and manual
//! If-Match checks. Updates carry no If-Match equivalent and are
//! last-write-wins, like Microsoft Graph.

use std::{
    collections::BTreeMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use io_jmap::rfc9610::contact_card::JmapContactCard;
use serde_json::{Map, Value, to_string};
use vcard::{tree::cst::VcardCst, vcard::Vcard};

use crate::shared::card::Card;

/// JMAP ContactCard to the shared card shape: the projected vCard
/// document as contents, the ContactCard id as id and the JSON hash as
/// ETag.
pub fn to_card(addressbook_id: &str, card: JmapContactCard) -> Result<Card, String> {
    let etag = etag(&card);
    let vcard = to_vcard(&card.card)?;
    let id = card
        .id
        .ok_or_else(|| "JMAP ContactCard is missing its id".to_string())?;

    Ok(Card {
        id,
        addressbook_id: addressbook_id.to_string(),
        etag,
        contents: vcard.into_bytes(),
    })
}

/// Projects the JSContact Card properties onto a vCard document.
pub fn to_vcard(card: &Map<String, Value>) -> Result<String, String> {
    let json = Value::Object(card.clone());
    let vcard =
        Vcard::from_jscontact(&json).map_err(|err| format!("Invalid JSContact card: {err}"))?;

    Ok(vcard.to_string())
}

/// Projects a vCard document onto JSContact Card properties, the
/// create payload of `ContactCard/set`.
pub fn to_jscontact(vcard: &str) -> Result<Map<String, Value>, String> {
    let cst = VcardCst::parse(vcard).map_err(|err| format!("Invalid vCard: {err}"))?;

    match cst.decode().to_jscontact() {
        Value::Object(map) => Ok(map),
        _ => Err("JSContact conversion did not produce a card object".to_string()),
    }
}

/// `ContactCard/set` update patch from the edited vCard: each
/// top-level JSContact property that differs from the base vCard (the
/// state last synced with the server), plus a null for every property
/// the edit removed. Without a base the patch carries every property,
/// which cannot clear server-side ones the vCard lost track of.
pub fn to_patch(vcard: &str, base_vcard: Option<&str>) -> Result<BTreeMap<String, Value>, String> {
    let new = to_jscontact(vcard)?;
    let mut patch = BTreeMap::new();

    match base_vcard {
        Some(base) => {
            let base = to_jscontact(base)?;

            for (key, value) in &new {
                if base.get(key) != Some(value) {
                    patch.insert(key.clone(), value.clone());
                }
            }

            for key in base.keys() {
                // NOTE: the uid is immutable in spirit (RFC 9610 §3
                // keys groups on it); never null it out.
                if !new.contains_key(key) && key != "uid" {
                    patch.insert(key.clone(), Value::Null);
                }
            }
        }
        None => patch.extend(new),
    }

    // NOTE: the JMAP envelope is not part of the JSContact payload;
    // addressBookIds in particular must survive the update.
    patch.remove("id");
    patch.remove("addressBookIds");

    Ok(patch)
}

/// Revision token of a ContactCard: a hash of its JSON. serde_json
/// maps are key-sorted, so the hash is independent of the property
/// order the server picked.
fn etag(card: &JmapContactCard) -> Option<String> {
    let json = to_string(card).ok()?;
    let mut hasher = DefaultHasher::new();
    json.hash(&mut hasher);

    Some(format!("{:016x}", hasher.finish()))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const VCARD: &str = "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:abc\r\nFN:Jane Doe\r\nEMAIL:jane@example.com\r\nTEL:+33612345678\r\nEND:VCARD\r\n";

    #[test]
    fn vcard_to_jscontact_maps_core_properties() {
        let card = to_jscontact(VCARD).unwrap();

        assert_eq!(
            card.get("uid").and_then(|uid| uid.as_str()),
            Some("abc"),
            "{card:?}"
        );
        assert!(card.contains_key("name"), "{card:?}");
        assert!(card.contains_key("emails"), "{card:?}");
        assert!(card.contains_key("phones"), "{card:?}");
    }

    #[test]
    fn to_jscontact_emits_no_non_standard_vcard_container() {
        let card = to_jscontact(VCARD).unwrap();

        // vcard-rs preserves unmapped properties under the standard
        // `vCardProps` member; it never emits calcard's non-standard
        // top-level `vCard` object that strict servers reject.
        assert!(!card.contains_key("vCard"), "{card:?}");
    }

    #[test]
    fn jscontact_to_vcard_round_trips() {
        let card = to_jscontact(VCARD).unwrap();
        let vcard = to_vcard(&card).unwrap();

        assert!(vcard.contains("FN:Jane Doe"), "{vcard}");
        assert!(vcard.contains("UID:abc"), "{vcard}");
        assert!(vcard.contains("jane@example.com"), "{vcard}");
    }

    #[test]
    fn patch_without_base_carries_every_property() {
        let patch = to_patch(VCARD, None).unwrap();

        assert!(patch.contains_key("name"), "{patch:?}");
        assert!(patch.contains_key("emails"), "{patch:?}");
        assert!(!patch.contains_key("id"), "{patch:?}");
        assert!(!patch.contains_key("addressBookIds"), "{patch:?}");
    }

    #[test]
    fn patch_against_base_keeps_only_changes() {
        let edited = VCARD.replace("Jane Doe", "Jane Smith");
        let patch = to_patch(&edited, Some(VCARD)).unwrap();

        assert!(patch.contains_key("name"), "{patch:?}");
        assert!(!patch.contains_key("emails"), "{patch:?}");
        assert!(!patch.contains_key("phones"), "{patch:?}");
        assert!(!patch.contains_key("uid"), "{patch:?}");
    }

    #[test]
    fn patch_nulls_removed_properties() {
        let edited = VCARD.replace("TEL:+33612345678\r\n", "");
        let patch = to_patch(&edited, Some(VCARD)).unwrap();

        assert_eq!(patch.get("phones"), Some(&Value::Null));
        assert!(!patch.contains_key("uid"), "{patch:?}");
    }

    #[test]
    fn middle_name_rides_given2_and_round_trips() {
        // NOTE: RFC 9553 carries the middle name as the given2
        // component kind; folding it into given (or dropping it) would
        // destroy N's third component through every JMAP round-trip.
        let vcard = "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:abc\r\nN:BB;Aa;g;;\r\nEND:VCARD\r\n";
        let card = to_jscontact(vcard).unwrap();

        let components = card
            .get("name")
            .and_then(|name| name.get("components"))
            .and_then(|components| components.as_array())
            .expect("name components");
        let given2 = components.iter().any(|component| {
            component.get("kind").and_then(|kind| kind.as_str()) == Some("given2")
                && component.get("value").and_then(|value| value.as_str()) == Some("g")
        });
        assert!(given2, "{card:?}");

        let round = to_vcard(&card).unwrap();
        assert!(round.contains("N:BB;Aa;g;;"), "{round}");
    }

    #[test]
    fn name_components_without_full_mint_no_display_name() {
        // NOTE: a card with name components but no name.full (display
        // name never set) must convert without an FN: the app never
        // mints one into the record, the lists compose on the fly.
        let card = json!({
            "@type": "Card",
            "version": "1.0",
            "uid": "abc",
            "name": {
                "components": [
                    { "kind": "given", "value": "Jane" },
                    { "kind": "surname", "value": "Doe" },
                ],
            },
        });

        let vcard = to_vcard(card.as_object().unwrap()).unwrap();

        assert!(!vcard.contains("FN"), "{vcard}");
        assert!(vcard.contains("Jane"), "{vcard}");
    }
}
