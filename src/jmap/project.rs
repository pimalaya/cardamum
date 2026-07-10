//! JMAP ContactCard (RFC 9610) to vCard projection and back, via
//! calcard's JSContact conversion (RFC 9555). The ContactCard's
//! JSContact payload (RFC 9553) converts losslessly enough for the
//! app's purposes: unmapped vCard properties ride in vCardProps, so
//! the vCard document of record round-trips.
//!
//! JMAP has no per-card ETag; the revision surfaced by [`to_card`] is
//! a hash of the card's JSON, which only drives display and manual
//! If-Match checks. Updates carry no If-Match equivalent and are
//! last-write-wins, like Microsoft Graph.

use std::{
    collections::BTreeMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use calcard::{jscontact::JSContact, vcard::VCard};
use io_jmap::rfc9610::contact_card::JmapContactCard;

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
pub fn to_vcard(card: &serde_json::Map<String, serde_json::Value>) -> Result<String, String> {
    let json = serde_json::Value::Object(card.clone()).to_string();
    let jscontact: JSContact<String, String> =
        JSContact::parse(&json).map_err(|err| format!("Invalid JSContact card: {err}"))?;
    let vcard = jscontact
        .into_vcard()
        .ok_or_else(|| "JSContact card does not convert to a vCard".to_string())?;

    Ok(vcard.to_string())
}

/// Projects a vCard document onto JSContact Card properties, the
/// create payload of `ContactCard/set`.
pub fn to_jscontact(vcard: &str) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    let vcard = VCard::parse(vcard).map_err(|_| "Invalid vCard".to_string())?;
    let jscontact: JSContact<String, String> = vcard.into_jscontact();
    let value = serde_json::to_value(&jscontact.0)
        .map_err(|err| format!("Unserializable JSContact card: {err}"))?;

    match value {
        serde_json::Value::Object(map) => Ok(map),
        _ => Err("JSContact conversion did not produce a card object".to_string()),
    }
}

/// `ContactCard/set` update patch from the edited vCard: each
/// top-level JSContact property that differs from the base vCard (the
/// state last synced with the server), plus a null for every property
/// the edit removed. Without a base the patch carries every property,
/// which cannot clear server-side ones the vCard lost track of.
pub fn to_patch(
    vcard: &str,
    base_vcard: Option<&str>,
) -> Result<BTreeMap<String, serde_json::Value>, String> {
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
                    patch.insert(key.clone(), serde_json::Value::Null);
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
    let json = serde_json::to_string(card).ok()?;
    let mut hasher = DefaultHasher::new();
    json.hash(&mut hasher);

    Some(format!("{:016x}", hasher.finish()))
}

#[cfg(test)]
mod tests {
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

        assert_eq!(patch.get("phones"), Some(&serde_json::Value::Null));
        assert!(!patch.contains_key("uid"), "{patch:?}");
    }
}
