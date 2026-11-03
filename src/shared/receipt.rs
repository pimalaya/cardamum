//! # Receipts
//!
//! What a write prints when it has no payload of its own to show: the
//! item it acted on, by id, and for a shared write whether a sync engine
//! applies it later.
//!
//! The keys keep the meanings the machine output contract gives them:
//! `id` names the item as it is after the write, `queued` says whether
//! the write waits for a sync. The sentence of the text form is not data,
//! so it stays out of the JSON.

use std::fmt;

use schemars::JsonSchema;
use serde::Serialize;

/// The receipt of a shared write acting on one item.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WriteReceipt {
    /// The id of the item acted on, as it is after the write.
    pub id: String,
    /// Whether a sync engine applies the write later rather than now.
    pub queued: bool,
    /// What the text form names, as in `Card`.
    #[serde(skip)]
    pub what: &'static str,
    /// The write made, which the text form words.
    #[serde(skip)]
    pub write: Write,
}

impl fmt::Display for WriteReceipt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            id, what, write, ..
        } = self;
        match self.queued {
            true => writeln!(
                f,
                "{what} `{id}` {} queued; the next sync applies it",
                write.noun()
            ),
            false => writeln!(f, "{what} `{id}` successfully {}", write.past()),
        }
    }
}

/// A write a [`WriteReceipt`] reports.
#[derive(Clone, Copy, Debug)]
pub enum Write {
    /// An update.
    Update,
    /// A deletion.
    Delete,
}

impl Write {
    /// The write as a noun, as in `deletion`.
    fn noun(self) -> &'static str {
        match self {
            Self::Update => "update",
            Self::Delete => "deletion",
        }
    }

    /// The write done, as in `deleted`.
    fn past(self) -> &'static str {
        match self {
            Self::Update => "updated",
            Self::Delete => "deleted",
        }
    }
}

/// The receipt of a write in a protocol namespace acting on one item.
#[cfg(any(
    feature = "carddav",
    feature = "gpeople",
    feature = "jmap",
    feature = "msgraph",
    feature = "vdir"
))]
#[derive(Debug, Serialize, JsonSchema)]
pub struct IdReceipt {
    /// The id of the item acted on, as it is after the write.
    pub id: String,
    /// What the text form says was done, as in `Rule created`.
    #[serde(skip)]
    pub done: &'static str,
}

#[cfg(any(
    feature = "carddav",
    feature = "gpeople",
    feature = "jmap",
    feature = "msgraph",
    feature = "vdir"
))]
impl fmt::Display for IdReceipt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}: {}", self.done, self.id)
    }
}

/// The receipt of a write in a protocol namespace acting on several items.
#[cfg(feature = "jmap")]
#[derive(Debug, Serialize, JsonSchema)]
pub struct IdsReceipt {
    /// The ids of the items acted on, in the order acted on.
    pub ids: Vec<String>,
    /// What the text form says was done, as in `ContactCards copied`.
    #[serde(skip)]
    pub done: &'static str,
}

#[cfg(feature = "jmap")]
impl fmt::Display for IdsReceipt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}: {}", self.done, self.ids.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use crate::shared::receipt::{Write, WriteReceipt};

    #[test]
    fn the_sentence_stays_out_of_the_json() {
        let receipt = WriteReceipt {
            id: "a.vcf".into(),
            queued: true,
            what: "Card",
            write: Write::Delete,
        };

        assert_eq!(
            receipt.to_string(),
            "Card `a.vcf` deletion queued; the next sync applies it\n"
        );
        assert_eq!(
            serde_json::to_string(&receipt).unwrap(),
            r#"{"id":"a.vcf","queued":true}"#
        );
    }
}
