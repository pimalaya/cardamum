//! # vCard source
//!
//! Where the card commands get raw vCard bytes: a file, the standard
//! input, the argument itself, or nothing at all.

use std::{
    fs,
    io::{Read, stdin},
    path::PathBuf,
};

use anyhow::{Context, Result, bail};
use clap::ValueEnum;
use vcard::{
    builder::VcardBuilder,
    prop::uid::UID,
    tree::cst::VcardCst,
    value::{VcardValue, uri::VcardUri},
    version::VcardVersion,
};

use crate::shared::uuid::uuid_v4;

/// Resolves a vCard source into raw bytes.
///
/// `-` reads stdin and an existing path is read from disk, otherwise the
/// value is taken as literal vCard contents.
pub fn read_source(source: &str) -> Result<Vec<u8>> {
    if source == "-" {
        let mut buf = Vec::new();
        stdin()
            .read_to_end(&mut buf)
            .context("Read vCard from stdin error")?;
        return Ok(buf);
    }

    let path = PathBuf::from(source);

    if path.is_file() {
        return fs::read(&path)
            .with_context(|| format!("Read vCard from `{}` error", path.display()));
    }

    if source.trim_start().starts_with("BEGIN:VCARD") {
        return Ok(source.as_bytes().to_vec());
    }

    bail!("Source `{source}` is neither a readable file nor vCard contents")
}

/// What is wrong with a card, empty when nothing is.
///
/// Checks it against its own version's RFC contract through vcard-rs, so
/// a 4.0 card missing its required `FN` is caught here rather than by the
/// server, or worse by neither. Reading is liberal and this is the strict
/// half: bytes that do not parse at all come back as the one violation
/// they are.
pub fn check(card: &[u8]) -> Vec<String> {
    let cst = match VcardCst::parse(card) {
        Ok(cst) => cst,
        Err(err) => return vec![format!("{err}")],
    };

    match cst.decode().validate() {
        Ok(_) => Vec::new(),
        Err(errors) => errors.iter().map(|err| format!("{err}")).collect(),
    }
}

/// Mints a card carrying nothing but its identity.
///
/// This is what a create starts from when it is given no source, and it
/// is a card rather than an empty file so that no composer is asked to
/// invent a `UID`: the pimdir link id derives from it, and an editor
/// handed an empty file mints none.
pub fn blank_card(version: VcardVersion) -> Result<Vec<u8>> {
    let uid = format!("urn:uuid:{}", uuid_v4()?);

    let card = VcardBuilder::new(version)
        .prop::<UID>()
        .value(VcardValue::Uri(VcardUri(uid.into())))
        .build_unchecked();

    Ok(card.encode().to_bytes())
}

/// vCard versions a minted card can be written at.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum CardVersionArg {
    /// vCard 2.1, the pre-standard version older exporters still write.
    #[value(name = "2.1")]
    V2_1,
    /// vCard 3.0, RFC 2426.
    #[value(name = "3.0")]
    V3_0,
    /// vCard 4.0, RFC 6350.
    #[value(name = "4.0")]
    V4_0,
}

impl From<CardVersionArg> for VcardVersion {
    fn from(version: CardVersionArg) -> Self {
        match version {
            CardVersionArg::V2_1 => Self::V2_1,
            CardVersionArg::V3_0 => Self::V3_0,
            CardVersionArg::V4_0 => Self::V4_0,
        }
    }
}
