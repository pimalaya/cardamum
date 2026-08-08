//! The `text/vcard` derivations a pimdir store records for a card: its
//! link id, its `v: 1` summary and its sort key (pimdir SPEC §9.3, §13).
//!
//! These MUST agree byte for byte with Neverest's `text/vcard` kind
//! (neverest/src/kind/vcard.rs), because both write into the same store: a
//! card Cardamum adds has to link, summarize and sort exactly as the same card
//! arriving through a sync, or the store holds it twice.
//!
//! Cards are read with a deliberately small scanner rather than a vCard
//! parser, mirroring Neverest. Only the `UID` and a handful of display fields
//! are needed, and the body must never be rewritten: a card reaches the store
//! as opaque bytes, byte for byte, so a property this scanner does not
//! understand cannot be lost. The full parser (vcard-rs) stays on the
//! rendering side.

use io_replica::placement::{ReplicaLinkId, ReplicaMeta, ReplicaSortKey};
use serde::{Deserialize, Serialize};

/// The `text/vcard` meta schema version this writer emits (pimdir SPEC §13).
const META_VERSION: u8 = 1;

/// Versioned card summary persisted as the `text/vcard` [`ReplicaMeta`] blob,
/// the stable JSON contract a reader parses to render a contact list without
/// fetching a body. Absent optional fields mean "unknown".
#[derive(Default, Deserialize, Serialize)]
pub struct CardSummary {
    /// Schema version ([`META_VERSION`]).
    pub v: u8,
    /// The card's `UID`, when it carries one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uid: Option<String>,
    /// The card's display name (`FN`), empty when the card carries none.
    #[serde(default, rename = "fn")]
    pub full_name: String,
    /// Every `EMAIL` value, in document order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub emails: Vec<String>,
    /// The body's size in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

/// The whole `Full`-tier derivation of a raw vCard: link id, summary and sort
/// key, the three things a staged write has to supply.
pub fn derive(raw: &[u8]) -> (ReplicaLinkId, ReplicaMeta, ReplicaSortKey) {
    let props = properties(raw);

    let uid = first(&props, "UID");
    let full_name = first(&props, "FN").unwrap_or_default();
    let emails: Vec<String> = props
        .iter()
        .filter(|(name, value)| name == "EMAIL" && !value.is_empty())
        .map(|(_, value)| value.clone())
        .collect();

    let link_id = link_id(uid.as_deref(), raw);
    let sort_key = sort_key(&full_name);
    let summary = CardSummary {
        v: META_VERSION,
        uid,
        full_name,
        emails,
        size: Some(raw.len() as u64),
    };
    let meta = ReplicaMeta(serde_json::to_string(&summary).unwrap_or_default());

    (link_id, meta, sort_key)
}

/// The link id for a card: its `UID` (`uid:`), else a digest of the whole
/// body (`hash:`).
///
/// A card without a `UID` is legal but unaddressable across servers, so the
/// fallback only has to be stable and collision-resistant enough to keep two
/// different cards apart within one store. FNV-1a is used rather than
/// `DefaultHasher` because its output is fixed by its own specification: a
/// hash that changed with a compiler version would silently re-link every
/// such card and store its body twice.
fn link_id(uid: Option<&str>, raw: &[u8]) -> ReplicaLinkId {
    match uid {
        Some(uid) if !uid.is_empty() => ReplicaLinkId::from(format!("uid:{uid}")),
        _ => ReplicaLinkId::from(format!("hash:{:016x}", fnv1a64(raw))),
    }
}

/// The contacts ordering key (pimdir SPEC §9.3): the display name, case-folded
/// and trimmed so `alice` and `Alice` sort together. A card with no `FN` yields
/// the empty key, which the store reads as unknown.
pub fn sort_key(full_name: &str) -> ReplicaSortKey {
    ReplicaSortKey::from(full_name.trim().to_lowercase())
}

/// FNV-1a, 64-bit (the reference offset basis and prime).
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// The first value of `name`, trimmed.
fn first(props: &[(String, String)], name: &str) -> Option<String> {
    props
        .iter()
        .find(|(prop, _)| prop == name)
        .map(|(_, value)| value.clone())
}

/// Splits a raw card into `(NAME, value)` pairs: lines unfolded (RFC 6350
/// §3.2), group prefixes and parameters stripped from the name, the name
/// upper-cased, and the value unescaped.
///
/// Bytes are read lossily on purpose: a card whose encoding a server got wrong
/// still yields its `UID`, and the body itself is stored untouched either way.
fn properties(raw: &[u8]) -> Vec<(String, String)> {
    let text = String::from_utf8_lossy(raw);
    let mut logical: Vec<String> = Vec::new();

    for line in text.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        match line.strip_prefix([' ', '\t']) {
            // A line starting with one whitespace continues the previous one,
            // that whitespace being the fold marker rather than content.
            Some(rest) => {
                if let Some(last) = logical.last_mut() {
                    last.push_str(rest);
                    continue;
                }
                logical.push(rest.to_owned());
            }
            None => logical.push(line.to_owned()),
        }
    }

    logical
        .iter()
        .filter_map(|line| split_property(line))
        .collect()
}

/// Splits one unfolded line into its upper-cased name and unescaped value, or
/// `None` when it carries no unquoted `:` (a blank line, `BEGIN`-less noise).
fn split_property(line: &str) -> Option<(String, String)> {
    let colon = unquoted_colon(line)?;
    let (head, value) = line.split_at(colon);

    // Parameters (`;TYPE=work`) and a group prefix (`item1.EMAIL`) both
    // decorate the name; neither is part of it.
    let name = head.split(';').next().unwrap_or(head);
    let name = name.rsplit('.').next().unwrap_or(name);

    Some((
        name.trim().to_uppercase(),
        unescape(value[1..].trim_start()),
    ))
}

/// The byte index of the first `:` outside a quoted parameter value. A
/// parameter may legally hold one (`PHOTO;VALUE="uri:x":…`), and taking that
/// one would cut the property name in half.
fn unquoted_colon(line: &str) -> Option<usize> {
    let mut quoted = false;
    for (index, char) in line.char_indices() {
        match char {
            '"' => quoted = !quoted,
            ':' if !quoted => return Some(index),
            _ => {}
        }
    }
    None
}

/// Undoes the RFC 6350 §3.4 value escaping, for display fields only. The
/// stored body keeps its own escaping untouched.
fn unescape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();

    while let Some(char) = chars.next() {
        if char != '\\' {
            out.push(char);
            continue;
        }
        match chars.next() {
            Some('n' | 'N') => out.push('\n'),
            Some(escaped) => out.push(escaped),
            None => out.push('\\'),
        }
    }

    out.trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CARD: &str = "BEGIN:VCARD\r\n\
         VERSION:4.0\r\n\
         UID:urn:uuid:4fbe8971-0bc3-424c-9c26-36c3e1eff6b1\r\n\
         FN:Jane Doe\r\n\
         item1.EMAIL;TYPE=work:jane@example.org\r\n\
         EMAIL;TYPE=home:jane@home.example.org\r\n\
         END:VCARD\r\n";

    fn summary(raw: &str) -> serde_json::Value {
        let (_, meta, _) = derive(raw.as_bytes());
        serde_json::from_str(&meta.0).unwrap()
    }

    #[test]
    fn the_uid_is_the_link_id_even_when_it_holds_colons() {
        // A `urn:uuid:` UID is the common shape, and it is exactly the case a
        // naive "split on the first colon" would mangle.
        let (link, _, _) = derive(CARD.as_bytes());
        assert_eq!(link.0, "uid:urn:uuid:4fbe8971-0bc3-424c-9c26-36c3e1eff6b1");
    }

    #[test]
    fn a_card_without_a_uid_links_by_a_stable_digest() {
        let raw = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:No Uid\r\nEND:VCARD\r\n";
        let other = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Other One\r\nEND:VCARD\r\n";

        let (link, _, _) = derive(raw.as_bytes());
        let (again, _, _) = derive(raw.as_bytes());
        let (different, _, _) = derive(other.as_bytes());

        assert!(link.0.starts_with("hash:"), "got {}", link.0);
        assert_eq!(link, again, "the same body must link the same way twice");
        assert_ne!(link, different);
    }

    #[test]
    fn folded_lines_are_unfolded_before_the_name_is_read() {
        // The fold splits `UID` mid-value and `FN` mid-name, both legal.
        let raw = "BEGIN:VCARD\r\n\
             VERSION:4.0\r\n\
             UID:card\r\n \
             -1234\r\n\
             F\r\n N:Jane Doe\r\n\
             END:VCARD\r\n";

        let (link, _, _) = derive(raw.as_bytes());
        assert_eq!(link.0, "uid:card-1234");
        assert_eq!(summary(raw)["fn"], "Jane Doe");
    }

    #[test]
    fn the_summary_carries_the_display_fields_of_every_email_shape() {
        let card = summary(CARD);

        assert_eq!(card["v"], 1);
        assert_eq!(card["fn"], "Jane Doe");
        assert_eq!(card["size"], CARD.len());
        // A group prefix (`item1.EMAIL`) names the same property.
        assert_eq!(
            card["emails"],
            serde_json::json!(["jane@example.org", "jane@home.example.org"])
        );
    }

    #[test]
    fn a_parameter_may_hold_a_colon_of_its_own() {
        let raw = "BEGIN:VCARD\r\n\
             VERSION:4.0\r\n\
             UID:card-1\r\n\
             FN;LANGUAGE=\"x:y\":Jane Doe\r\n\
             END:VCARD\r\n";

        assert_eq!(summary(raw)["fn"], "Jane Doe");
    }

    #[test]
    fn display_values_are_unescaped_but_the_body_is_not_touched() {
        let raw = "BEGIN:VCARD\r\n\
             VERSION:4.0\r\n\
             UID:card-1\r\n\
             FN:Doe\\, Jane\r\n\
             END:VCARD\r\n";

        assert_eq!(summary(raw)["fn"], "Doe, Jane");
    }

    #[test]
    fn a_card_with_bare_line_feeds_parses_like_a_crlf_one() {
        let lf = CARD.replace("\r\n", "\n");

        let (crlf_link, _, _) = derive(CARD.as_bytes());
        let (lf_link, _, _) = derive(lf.as_bytes());

        assert_eq!(crlf_link, lf_link);
        assert_eq!(summary(&lf)["fn"], "Jane Doe");
    }

    #[test]
    fn the_sort_key_case_folds_the_display_name() {
        let (_, _, key) = derive(CARD.as_bytes());
        assert_eq!(key.0, "jane doe");

        // No FN means no key, which the store reads as unknown.
        let raw = "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:x\r\nEND:VCARD\r\n";
        let (_, _, key) = derive(raw.as_bytes());
        assert!(key.is_unknown());
    }
}
