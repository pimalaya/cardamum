//! # vCard projection helpers
//!
//! The pieces every provider projection needs: canonical text
//! properties, date normalization, stash splicing and RFC 6350 escaping.
//!
//! Microsoft Graph and Google People expose no vCard of their own, so
//! their projections synthesize the document of record here. Ported from
//! cardamum-android, so both products treat the same quirks identically.

use std::borrow::Cow;

use vcard::{
    param::VcardParam,
    prop::{VcardProp, VcardPropKind, VcardPropName},
    value::{VcardValue, text::VcardText},
};

/// Longest raw property line the provider backends stash server-side.
///
/// A longer line, a base64 PHOTO blob essentially, stays in the local
/// document of record rather than risking the whole write against an
/// undocumented provider size limit.
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

/// Normalizes a BDAY value to `yyyy-mm-dd`, `None` when partial.
///
/// A year-less date has no standard vCard 3 form, so it does not sync.
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

/// Splices raw logical property lines, endings excluded, into a
/// serialized vCard right before its END:VCARD line.
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

/// Escapes a text value for a minted property line, per RFC 6350
/// section 3.4: backslash, comma, semicolon and newline.
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
