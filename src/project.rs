//! Helpers shared by the vCard projections of the API backends
//! (Microsoft Graph, Google People), ported from cardamum-android.
//!
//! The API backends expose no vCard representation of a contact, so
//! their projections synthesize the vCard document of record
//! themselves; these helpers cover the pieces every projection needs
//! (canonical text properties, date normalization, stash splicing and
//! RFC 6350 text escaping).

use std::borrow::Cow;

use vcard::{
    param::VcardParam,
    prop::{VcardProp, VcardPropKind, VcardPropName},
    value::{VcardValue, text::VcardText},
};

/// Longest raw property line the provider backends stash server-side.
/// Longer lines (base64 PHOTO blobs, essentially) stay only in the
/// local document of record instead of risking the whole write against
/// undocumented provider size limits.
pub const MAX_STASH_LINE: usize = 8 * 1024;

/// A canonical text property built from an owned value.
pub fn text_prop(
    kind: VcardPropKind,
    params: Vec<VcardParam<'static>>,
    value: &str,
) -> VcardProp<'static> {
    VcardProp {
        name: VcardPropName::Kind(kind),
        params,
        value: VcardValue::Text(VcardText(Cow::Owned(value.to_string()))),
    }
}

/// Normalizes a BDAY value to `yyyy-mm-dd`, or None for anything partial
/// (year-less dates have no standard vCard 3 form, so they do not sync).
pub fn full_date(raw: &str) -> Option<String> {
    let date = raw.trim();
    let digits = |s: &str| s.bytes().all(|b| b.is_ascii_digit());

    let dashed: Vec<&str> = date.split('-').collect();
    if let [y, m, d] = dashed[..]
        && y.len() == 4
        && m.len() == 2
        && d.len() == 2
        && digits(y)
        && digits(m)
        && digits(d)
    {
        return Some(format!("{y}-{m}-{d}"));
    }

    if date.len() == 8 && digits(date) {
        return Some(format!("{}-{}-{}", &date[..4], &date[4..6], &date[6..]));
    }

    None
}

/// Splices raw property lines (logical lines without their ending)
/// into a serialized vCard, right before its END:VCARD line.
pub fn splice_props(vcard: String, lines: &[String]) -> String {
    if lines.is_empty() {
        return vcard;
    }

    let mut extra = lines.join("\r\n");
    extra.push_str("\r\n");

    match vcard.rfind("END:VCARD") {
        Some(position) => {
            let mut out = vcard;
            out.insert_str(position, &extra);
            out
        }
        None => vcard + &extra,
    }
}

/// Escapes a text value for a minted property line (RFC 6350 3.4:
/// backslash, comma, semicolon and newline).
pub fn escape_text(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\\' => out.push_str("\\\\"),
            ',' => out.push_str("\\,"),
            ';' => out.push_str("\\;"),
            '\n' => out.push_str("\\n"),
            '\r' => {}
            _ => out.push(character),
        }
    }
    out
}
