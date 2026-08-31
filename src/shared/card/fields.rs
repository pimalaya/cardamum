//! # Card fields
//!
//! The vCard properties a create or an update can set from the command
//! line, and how they are written onto a card.
//!
//! They are a convenience over the common fields rather than the whole of
//! vCard: the composer is the complete surface, which is why `ADR` and its
//! seven components are deliberately not a flag.

use std::borrow::Cow;

use anyhow::{Context, Result, bail};
use clap::Parser;
use vcard::{
    param::VcardParam,
    prop::{VcardProp, VcardPropKind, VcardPropName},
    tree::{codec::mode::VcardEscaper, cst::VcardCst, line::VcardLine},
    value::{
        VcardValue,
        datetime::VcardDateAndOrTime,
        n::VcardN,
        org::VcardOrg,
        text::{VcardText, VcardTextList},
        uri::VcardUri,
    },
};

/// The vCard fields `card create` and `card update` set from flags.
///
/// Each flag names one property, and writing it replaces every instance
/// of that property the card already carried.
#[derive(Debug, Parser)]
pub struct CardFieldsArgs {
    /// Identity of the card (`UID`).
    ///
    /// A card minted from nothing gets a fresh `urn:uuid` when this names
    /// none, the pimdir link id deriving from it.
    #[arg(long, value_name = "TEXT")]
    pub uid: Option<String>,
    /// Display name (`FN`), which every vCard is required to carry.
    #[arg(long, value_name = "TEXT")]
    pub full_name: Option<String>,
    /// Given names (`N`), the flag repeating, the other components left
    /// as they were.
    ///
    /// Named for the role rather than the position: which of the two is
    /// written first is what varies between cultures.
    #[arg(long, value_name = "TEXT")]
    pub given_name: Vec<String>,
    /// Family names (`N`), the flag repeating, the other components left
    /// as they were.
    #[arg(long, value_name = "TEXT")]
    pub family_name: Vec<String>,
    /// Middle names (`N` additional), the flag repeating.
    #[arg(long, value_name = "TEXT")]
    pub middle_name: Vec<String>,
    /// Honorific prefixes (`N`), the flag repeating: Dr., Mr.
    #[arg(long, value_name = "TEXT")]
    pub name_prefix: Vec<String>,
    /// Honorific suffixes (`N`), the flag repeating: Jr., PhD.
    #[arg(long, value_name = "TEXT")]
    pub name_suffix: Vec<String>,
    /// Nicknames (`NICKNAME`), the flag repeating.
    #[arg(long, value_name = "TEXT")]
    pub nickname: Vec<String>,
    /// Organization (`ORG`), the flag repeating for its units.
    #[arg(long, value_name = "TEXT")]
    pub organization: Vec<String>,
    /// Job title (`TITLE`).
    #[arg(long, value_name = "TEXT")]
    pub title: Option<String>,
    /// Birthday (`BDAY`), in the RFC 6350 basic form `19960415`.
    #[arg(long, value_name = "DATE")]
    pub birthday: Option<String>,
    /// Email addresses (`EMAIL`), the flag repeating.
    ///
    /// A `work:` or `home:` prefix becomes the `TYPE` parameter.
    #[arg(long, value_name = "[TYPE:]ADDRESS")]
    pub email: Vec<String>,
    /// Phone numbers (`TEL`), the flag repeating, taking the same
    /// optional `TYPE` prefix as `--email`.
    #[arg(long, value_name = "[TYPE:]NUMBER")]
    pub phone: Vec<String>,
    /// Web pages (`URL`), the flag repeating.
    #[arg(long, value_name = "URL")]
    pub url: Vec<String>,
    /// Free-form note (`NOTE`).
    #[arg(long, value_name = "TEXT")]
    pub note: Option<String>,
}

impl CardFieldsArgs {
    /// Whether no field flag was given, the card then passing through.
    pub fn is_empty(&self) -> bool {
        self.uid.is_none()
            && self.full_name.is_none()
            && self.given_name.is_empty()
            && self.family_name.is_empty()
            && self.middle_name.is_empty()
            && self.name_prefix.is_empty()
            && self.name_suffix.is_empty()
            && self.nickname.is_empty()
            && self.organization.is_empty()
            && self.title.is_none()
            && self.birthday.is_none()
            && self.email.is_empty()
            && self.phone.is_empty()
            && self.url.is_empty()
            && self.note.is_none()
    }

    /// Writes the flags onto `card`, returning its new bytes.
    ///
    /// Every instance of a property a flag names is dropped and the flag's
    /// own is appended, so a flag sets that property rather than adding to
    /// it. Every other line keeps the card's own bytes, the properties no
    /// flag covers included.
    pub fn apply(&self, card: &[u8]) -> Result<Vec<u8>> {
        if self.is_empty() {
            return Ok(card.to_vec());
        }

        // NOTE: a flag rewrites one card and the parser reads the first,
        // so a file holding several would come back as its first card
        // alone. Dropping the rest silently is the one outcome worth
        // refusing outright.
        if VcardCst::parse_many(card)
            .filter(|card| card.is_ok())
            .count()
            > 1
        {
            bail!("Cannot apply a field flag to a source holding several vCards");
        }

        let mut cst = VcardCst::parse(card).context("Parse vCard error")?;
        let escaper = VcardEscaper::for_version_str(&version(&cst));
        let props = self.props(&cst);

        let written: Vec<VcardPropKind> = props
            .iter()
            .filter_map(|prop| match prop.name {
                VcardPropName::Kind(kind) => Some(kind),
                VcardPropName::Unknown(_) => None,
            })
            .collect();

        cst.props
            .retain(|line| !written.iter().any(|kind| names(line, *kind)));
        cst.props
            .extend(props.iter().map(|prop| prop.encode(escaper)));

        Ok(cst.to_bytes())
    }

    /// The properties the flags name, in a stable order.
    ///
    /// `N` is read back from `card` first, so setting one of its
    /// components leaves the others as they were rather than clearing
    /// them. Every component is a list, as RFC 6350 section 6.2.2 has it.
    fn props(&self, card: &VcardCst) -> Vec<VcardProp<'static>> {
        let mut props = Vec::new();

        if let Some(uid) = &self.uid {
            props.push(prop(VcardPropKind::Uid, uri(uid)));
        }

        if let Some(full_name) = &self.full_name {
            props.push(prop(VcardPropKind::Fn, text(full_name)));
        }

        if self.names_any() {
            props.push(prop(VcardPropKind::N, VcardValue::N(self.name(card))));
        }

        if !self.nickname.is_empty() {
            let value = VcardValue::TextList(VcardTextList(owned(&self.nickname)));
            props.push(prop(VcardPropKind::Nickname, value));
        }

        if !self.organization.is_empty() {
            let value = VcardValue::Org(VcardOrg(owned(&self.organization)));
            props.push(prop(VcardPropKind::Org, value));
        }

        if let Some(title) = &self.title {
            props.push(prop(VcardPropKind::Title, text(title)));
        }

        if let Some(birthday) = &self.birthday {
            let value = VcardValue::DateAndOrTime(VcardDateAndOrTime(birthday.clone().into()));
            props.push(prop(VcardPropKind::Bday, value));
        }

        for email in &self.email {
            let (kind, address) = split_type(email);
            props.push(typed(VcardPropKind::Email, kind, text(address)));
        }

        for phone in &self.phone {
            let (kind, number) = split_type(phone);
            props.push(typed(VcardPropKind::Tel, kind, text(number)));
        }

        for url in &self.url {
            props.push(prop(VcardPropKind::Url, uri(url)));
        }

        if let Some(note) = &self.note {
            props.push(prop(VcardPropKind::Note, text(note)));
        }

        props
    }

    /// The `N` value the name flags describe, over the card's own.
    fn name(&self, card: &VcardCst) -> VcardN<'static> {
        let existing = card.props.iter().find(|line| names(line, VcardPropKind::N));

        let component = |index: usize| match existing {
            Some(line) => line
                .value
                .decode_component_list(index)
                .iter()
                .map(|value| Cow::Owned(value.to_string()))
                .collect(),
            None => Vec::new(),
        };

        // NOTE: a flag that was not passed leaves its component as the card
        // wrote it, so setting a family name does not clear the given one.
        let over = |flag: &[String], index: usize| match flag.is_empty() {
            true => component(index),
            false => owned(flag),
        };

        VcardN {
            family: over(&self.family_name, 0),
            given: over(&self.given_name, 1),
            additional: over(&self.middle_name, 2),
            prefixes: over(&self.name_prefix, 3),
            suffixes: over(&self.name_suffix, 4),
        }
    }

    /// Whether any of the five name flags was given.
    fn names_any(&self) -> bool {
        !(self.given_name.is_empty()
            && self.family_name.is_empty()
            && self.middle_name.is_empty()
            && self.name_prefix.is_empty()
            && self.name_suffix.is_empty())
    }
}

/// The card's declared version, `4.0` when it names none.
fn version(card: &VcardCst) -> String {
    card.props
        .iter()
        .find(|line| line.name.get().eq_ignore_ascii_case("VERSION"))
        .map(|line| line.value.decode().to_string())
        .unwrap_or_else(|| String::from("4.0"))
}

/// Whether a raw line carries the given property, its group prefix
/// ignored.
fn names(line: &VcardLine, kind: VcardPropKind) -> bool {
    let name = line.name.get();
    let bare = name.rsplit_once('.').map_or(name, |(_, bare)| bare);

    bare.eq_ignore_ascii_case(&kind)
}

/// A property with no parameter.
fn prop(kind: VcardPropKind, value: VcardValue<'static>) -> VcardProp<'static> {
    VcardProp {
        name: VcardPropName::Kind(kind),
        params: Vec::new(),
        value,
    }
}

/// A property carrying the `TYPE` a flag prefixed, when it gave one.
fn typed(
    kind: VcardPropKind,
    r#type: Option<&str>,
    value: VcardValue<'static>,
) -> VcardProp<'static> {
    let params = match r#type {
        Some(r#type) => vec![VcardParam::Type(vec![Cow::Owned(r#type.to_string())])],
        None => Vec::new(),
    };

    VcardProp {
        name: VcardPropName::Kind(kind),
        params,
        value,
    }
}

/// A single text value.
fn text(value: &str) -> VcardValue<'static> {
    VcardValue::Text(VcardText(Cow::Owned(value.to_string())))
}

/// A single URI value.
fn uri(value: &str) -> VcardValue<'static> {
    VcardValue::Uri(VcardUri(Cow::Owned(value.to_string())))
}

/// Owns a list of flag values for a list-valued property.
fn owned(values: &[String]) -> Vec<Cow<'static, str>> {
    values
        .iter()
        .map(|value| Cow::Owned(value.clone()))
        .collect()
}

/// Splits an optional `TYPE` prefix off a flag value.
///
/// Neither an email address nor a phone number carries a colon, so the
/// first one delimits a type when there is one.
fn split_type(value: &str) -> (Option<&str>, &str) {
    match value.split_once(':') {
        Some((r#type, rest)) => (Some(r#type), rest),
        None => (None, value),
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::CardFieldsArgs;

    #[derive(Parser)]
    struct Wrap {
        #[command(flatten)]
        fields: CardFieldsArgs,
    }

    fn apply(card: &str, args: &[&str]) -> String {
        let wrap = Wrap::try_parse_from([&["cardamum"], args].concat()).unwrap();
        let out = wrap.fields.apply(card.as_bytes()).unwrap();

        String::from_utf8(out).unwrap()
    }

    const CARD: &str = "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:urn:uuid:abc\r\n\
                        FN:Jane Doe\r\nN:Doe;Jane;Q;Dr.;PhD\r\n\
                        EMAIL;TYPE=work:jane@acme.example\r\n\
                        EMAIL;TYPE=home:jane@home.example\r\nEND:VCARD\r\n";

    #[test]
    fn no_flag_leaves_the_card_byte_for_byte() {
        assert_eq!(apply(CARD, &[]), CARD);
    }

    #[test]
    fn a_flag_replaces_every_instance_of_its_property() {
        let out = apply(CARD, &["--email", "work:new@acme.example"]);

        assert!(!out.contains("jane@acme.example"));
        assert!(!out.contains("jane@home.example"));
        assert_eq!(out.matches("EMAIL").count(), 1);
        assert!(out.contains("EMAIL;TYPE=work:new@acme.example"));
    }

    #[test]
    fn an_untouched_property_keeps_its_own_bytes() {
        let out = apply(CARD, &["--title", "Engineer"]);

        assert!(out.contains("FN:Jane Doe"));
        assert!(out.contains("N:Doe;Jane;Q;Dr.;PhD"));
        assert!(out.contains("TITLE:Engineer"));
    }

    #[test]
    fn setting_one_name_component_leaves_the_others() {
        let out = apply(CARD, &["--given-name", "Janet"]);

        assert!(out.contains("N:Doe;Janet;Q;Dr.;PhD"));
        assert_eq!(out.matches("\r\nN:").count(), 1);
    }

    #[test]
    fn a_name_flag_on_a_card_with_no_name_writes_only_what_it_was_given() {
        let card = "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:urn:uuid:abc\r\nEND:VCARD\r\n";
        let out = apply(card, &["--family-name", "Doe"]);

        assert!(out.contains("N:Doe"));
    }

    #[test]
    fn a_name_component_holds_every_value_the_flag_repeated() {
        // RFC 6350 6.2.2: each component is a comma-separated list, which
        // is why the flags repeat rather than take one value each.
        let out = apply(CARD, &["--given-name", "John", "--given-name", "Philip"]);

        assert!(out.contains("N:Doe;John,Philip;Q;Dr.;PhD"));
    }

    #[test]
    fn every_name_component_has_a_flag() {
        let out = apply(
            CARD,
            &[
                "--family-name",
                "Stevenson",
                "--given-name",
                "John",
                "--middle-name",
                "Philip",
                "--middle-name",
                "Paul",
                "--name-prefix",
                "Dr.",
                "--name-suffix",
                "Jr.",
                "--name-suffix",
                "M.D.",
            ],
        );

        assert!(out.contains("N:Stevenson;John;Philip,Paul;Dr.;Jr.,M.D."));
    }

    #[test]
    fn a_text_value_is_escaped_on_the_way_out() {
        let out = apply(CARD, &["--note", "one, two; three"]);

        assert!(out.contains("NOTE:one\\, two\\; three"));
    }

    #[test]
    fn an_untyped_flag_writes_no_type_parameter() {
        let out = apply(CARD, &["--phone", "+1-555-0100"]);

        assert!(out.contains("TEL:+1-555-0100"));
    }
}
