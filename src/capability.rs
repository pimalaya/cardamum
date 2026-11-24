//! # Capabilities
//!
//! What each backend serves of the unified API.
//!
//! A capability is named after the unified command it stands for,
//! `<noun>.<verb>` (`card.delete`, `addressbook.update`), or
//! `<noun>.<verb>.<argument>` for a flag of that command a backend may
//! not serve (`card.delete.if-match`), never in the store's vocabulary.
//! A backend declares a capability only when it gives the result every
//! other declarer gives; a different result stays protocol-specific.
//!
//! A verb a backend does not serve fails before connecting, with the
//! `capability-missing` code. The tables are plain data keyed by backend
//! name and compiled into every build, compiled-out backends included,
//! so the `--help` they annotate reads the same in every build.

use std::fmt;

use anyhow::Result;
use pimalaya_cli::{capability::CapabilityLevel, error::CodedError};

use crate::error::ErrorCode;

use Capability as C;
use Support::{Full, None as Not, Partial, Server};

/// A unified capability, named by [`Capability::name`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capability {
    /// Listing the addressbooks.
    AddressbookList,
    /// Creating an addressbook.
    AddressbookCreate,
    /// Choosing the id of an addressbook created.
    AddressbookCreateId,
    /// Renaming an addressbook.
    AddressbookUpdate,
    /// Deleting an addressbook and the cards it holds.
    AddressbookDelete,
    /// Setting an addressbook's description.
    AddressbookDescription,
    /// Setting an addressbook's colour.
    AddressbookColor,
    /// Listing an addressbook's cards.
    CardList,
    /// Reading a card.
    CardRead,
    /// Searching the cards with the pimdir query language.
    CardSearch,
    /// Ordering the hits of a search by relevance (`--sort relevance`).
    CardSearchRelevance,
    /// Adding a card to an addressbook.
    CardAdd,
    /// Replacing a card.
    CardUpdate,
    /// Deleting a card.
    CardDelete,
    /// Replacing a card only while it is at a given version.
    CardUpdateIfMatch,
    /// Deleting a card only while it is at a given version.
    CardDeleteIfMatch,
}

impl Capability {
    /// Every capability, in the order the listings print them.
    pub const ALL: [Self; 16] = [
        C::AddressbookList,
        C::AddressbookCreate,
        C::AddressbookCreateId,
        C::AddressbookUpdate,
        C::AddressbookDelete,
        C::AddressbookDescription,
        C::AddressbookColor,
        C::CardList,
        C::CardRead,
        C::CardSearch,
        C::CardSearchRelevance,
        C::CardAdd,
        C::CardUpdate,
        C::CardDelete,
        C::CardUpdateIfMatch,
        C::CardDeleteIfMatch,
    ];

    /// The capability's name: the unified command it stands for, plus the
    /// long flag when it is an argument of that command.
    pub fn name(self) -> &'static str {
        match self {
            Self::AddressbookList => "addressbook.list",
            Self::AddressbookCreate => "addressbook.create",
            Self::AddressbookCreateId => "addressbook.create.id",
            Self::AddressbookUpdate => "addressbook.update",
            Self::AddressbookDelete => "addressbook.delete",
            Self::AddressbookDescription => "addressbook.update.description",
            Self::AddressbookColor => "addressbook.update.color",
            Self::CardList => "card.list",
            Self::CardRead => "card.read",
            Self::CardSearch => "card.search",
            Self::CardSearchRelevance => "card.search.sort",
            Self::CardAdd => "card.add",
            Self::CardUpdate => "card.update",
            Self::CardDelete => "card.delete",
            Self::CardUpdateIfMatch => "card.update.if-match",
            Self::CardDeleteIfMatch => "card.delete.if-match",
        }
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// How well a backend serves a capability, pimdir's levels plus the one
/// the server decides.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Support {
    /// Served with the result every declarer gives.
    Full,
    /// Served, the detail saying what differs or is missing.
    Partial(&'static str),
    /// Served when the server allows it, the detail saying which do not;
    /// refused by the server otherwise.
    Server(&'static str),
    /// Not served, the detail saying why and what serves it instead.
    None(&'static str),
}

impl Support {
    /// The level, as the help and `account capability` print it.
    pub fn level(self) -> CapabilityLevel {
        match self {
            Self::Full => CapabilityLevel::Full,
            Self::Partial(_) => CapabilityLevel::Partial,
            Self::Server(_) => CapabilityLevel::Server,
            Self::None(_) => CapabilityLevel::None,
        }
    }

    /// What the level leaves out, `None` for a full one.
    pub fn detail(self) -> Option<&'static str> {
        match self {
            Self::Full => None,
            Self::Partial(detail) | Self::Server(detail) | Self::None(detail) => Some(detail),
        }
    }
}

/// One declared capability of a backend.
type Row = (Capability, Support);

/// The backends declaring capabilities, by the name `--backend` takes.
pub const BACKENDS: [&str; 6] = ["carddav", "jmap", "msgraph", "gpeople", "vdir", "pimdir"];

/// What `backend` declares, nothing for a name serving no unified verb.
pub fn declared(backend: &str) -> &'static [Row] {
    match backend {
        "carddav" => CARDDAV,
        "jmap" => JMAP,
        "msgraph" => MSGRAPH,
        "gpeople" => GPEOPLE,
        "vdir" => VDIR,
        "pimdir" => PIMDIR,
        _ => &[],
    }
}

/// How `backend` serves `capability`, `none` for a row it does not
/// declare.
pub fn support(backend: &str, capability: Capability) -> Support {
    declared(backend)
        .iter()
        .find(|(declared, _)| *declared == capability)
        .map_or(Not("not declared"), |(_, support)| *support)
}

/// Fails with `capability-missing` on the first of `capabilities` that
/// `backend` does not serve, naming it and why, before anything connects.
pub fn require(backend: &str, capabilities: &[Capability]) -> Result<()> {
    for capability in capabilities {
        if let Not(detail) = support(backend, *capability) {
            let message = format!("The {backend} backend does not serve `{capability}`: {detail}");
            return Err(CodedError::new(ErrorCode::CapabilityMissing, message).into());
        }
    }

    Ok(())
}

/// The capabilities a unified leaf reaches, flag-dependent ones included.
const LEAVES: &[(&str, &str, &[Capability])] = &[
    ("addressbook", "list", &[C::AddressbookList]),
    (
        "addressbook",
        "create",
        &[
            C::AddressbookCreate,
            C::AddressbookCreateId,
            C::AddressbookDescription,
            C::AddressbookColor,
        ],
    ),
    (
        "addressbook",
        "update",
        &[
            C::AddressbookUpdate,
            C::AddressbookDescription,
            C::AddressbookColor,
        ],
    ),
    ("addressbook", "delete", &[C::AddressbookDelete]),
    ("card", "list", &[C::CardList]),
    ("card", "read", &[C::CardRead]),
    ("card", "search", &[C::CardSearch, C::CardSearchRelevance]),
    ("card", "add", &[C::CardAdd]),
    ("card", "update", &[C::CardUpdate, C::CardUpdateIfMatch]),
    ("card", "delete", &[C::CardDelete, C::CardDeleteIfMatch]),
];

/// Whether `backend` serves the leaf `group leaf` at all, its first
/// capability being the verb itself. A leaf the tables do not list
/// reaches no backend, so it is served.
pub fn serves(backend: &str, group: &str, leaf: &str) -> bool {
    let leaf = LEAVES
        .iter()
        .find(|(known_group, known_leaf, _)| *known_group == group && *known_leaf == leaf);

    leaf.and_then(|(_, _, capabilities)| capabilities.first())
        .is_none_or(|capability| !matches!(support(backend, *capability), Not(_)))
}

/// The `--help` footer of `group leaf`: for each capability it reaches,
/// the backends declaring it by level, then what each detail says.
///
/// Generated from the tables, so it never drifts from what the commands
/// enforce. Empty for a leaf reaching no backend.
pub fn help(group: &str, leaf: &str) -> String {
    let Some((_, _, capabilities)) = LEAVES
        .iter()
        .find(|(known_group, known_leaf, _)| *known_group == group && *known_leaf == leaf)
    else {
        return String::new();
    };

    let mut help = String::from("Backends:");

    for capability in *capabilities {
        let mut levels: Vec<(CapabilityLevel, Vec<&str>)> = Vec::new();
        let mut details: Vec<(&str, Vec<&str>)> = Vec::new();

        for backend in BACKENDS {
            let support = support(backend, *capability);
            group_by(&mut levels, support.level(), backend);

            if let Some(detail) = support.detail() {
                group_by(&mut details, detail, backend);
            }
        }

        let levels: Vec<String> = levels
            .into_iter()
            .map(|(level, backends)| format!("{level} on {}", backends.join(", ")))
            .collect();

        help.push_str(&format!("\n  {capability}: {}", levels.join("; ")));

        for (detail, backends) in details {
            help.push_str(&format!("\n    {}: {detail}", backends.join(", ")));
        }
    }

    help
}

/// Pushes `item` under `key`, opening the group on first sight so the
/// groups keep the order they were met in.
fn group_by<'a, K: PartialEq>(groups: &mut Vec<(K, Vec<&'a str>)>, key: K, item: &'a str) {
    match groups.iter_mut().find(|(known, _)| *known == key) {
        Some((_, items)) => items.push(item),
        None => groups.push((key, vec![item])),
    }
}

/// Why a store derived from the server takes no addressbook write.
const PIMDIR_COLLECTION: &str =
    "the addressbook row is the sync's to write: change it on the server, then sync";

/// What a queued pimdir write leaves to the store.
const PIMDIR_QUEUED: &str = "queued for the next sync, and refused when a source of the store \
     does not support it";

/// Why a backend minting its own ids takes no `--id`.
const MINTED_ID: &str = "the server mints the id of an addressbook, so `--id` is refused";

/// Why a backend without versions takes no `--if-match`.
/// Why a backend with a server serves no card search yet.
const NO_SEARCH: &str = "not served yet: a search would list every card of the address books \
     in scope, which is to be measured first";

/// What a search matched over the cards themselves leaves out.
const SEARCH_MATCHED: &str = "every card of the address books in scope matched by cardamum, the \
     terms only a pimdir store answers refused";

/// Why only an index orders by relevance.
const NO_RELEVANCE: &str = "relevance is the pimdir index's own ranking";

const NO_VERSION: &str = "the backend exposes no version to compare, so `--if-match` is refused";

/// What a Graph version compared in a request of its own leaves open.
const CHANGE_KEY: &str = "compared with the contact's `changeKey` read just before the write, \
     so a write landing in between is not caught";

/// What a JMAP version compared in a request of its own leaves open.
const CARD_HASH: &str = "compared with a hash of the ContactCard read just before the write, \
     so a write landing in between is not caught";

/// CardDAV (RFC 6352).
const CARDDAV: &[Row] = &[
    (C::AddressbookList, Full),
    (
        C::AddressbookCreate,
        Server("the server may forbid creating an addressbook, as iCloud does"),
    ),
    (C::AddressbookCreateId, Full),
    (C::AddressbookUpdate, Full),
    (
        C::AddressbookDelete,
        Server("the server may forbid deleting an addressbook, as iCloud does"),
    ),
    (C::AddressbookDescription, Full),
    (C::AddressbookColor, Full),
    (C::CardList, Full),
    (C::CardRead, Full),
    (C::CardSearch, Not(NO_SEARCH)),
    (C::CardSearchRelevance, Not(NO_RELEVANCE)),
    (C::CardAdd, Full),
    (C::CardUpdate, Full),
    (C::CardDelete, Full),
    (C::CardUpdateIfMatch, Full),
    (
        C::CardDeleteIfMatch,
        Partial("sent as `If-Match`, which some servers ignore on a delete, as iCloud does"),
    ),
];

/// JMAP for Contacts (RFC 9610).
const JMAP: &[Row] = &[
    (C::AddressbookList, Full),
    (
        C::AddressbookCreate,
        Server("refused when the account states `mayCreateAddressBook: false`, as Fastmail's does"),
    ),
    (C::AddressbookCreateId, Not(MINTED_ID)),
    (
        C::AddressbookUpdate,
        Server("the server may forbid renaming an addressbook, as Fastmail does"),
    ),
    (C::AddressbookDelete, Full),
    (C::AddressbookDescription, Full),
    (
        C::AddressbookColor,
        Not("a JMAP AddressBook carries no colour"),
    ),
    (C::CardList, Full),
    (C::CardRead, Full),
    (C::CardSearch, Not(NO_SEARCH)),
    (C::CardSearchRelevance, Not(NO_RELEVANCE)),
    (C::CardAdd, Full),
    (C::CardUpdate, Full),
    (C::CardDelete, Full),
    (C::CardUpdateIfMatch, Partial(CARD_HASH)),
    (C::CardDeleteIfMatch, Partial(CARD_HASH)),
];

/// Microsoft Graph contacts.
const MSGRAPH: &[Row] = &[
    (C::AddressbookList, Full),
    (C::AddressbookCreate, Full),
    (C::AddressbookCreateId, Not(MINTED_ID)),
    (
        C::AddressbookUpdate,
        Partial("the default Contacts folder cannot be renamed"),
    ),
    (
        C::AddressbookDelete,
        Partial("the default Contacts folder cannot be deleted"),
    ),
    (
        C::AddressbookDescription,
        Not("a Graph contact folder carries no description"),
    ),
    (
        C::AddressbookColor,
        Not("a Graph contact folder carries no colour"),
    ),
    (C::CardList, Full),
    (C::CardRead, Full),
    (C::CardSearch, Not(NO_SEARCH)),
    (C::CardSearchRelevance, Not(NO_RELEVANCE)),
    (C::CardAdd, Full),
    (C::CardUpdate, Full),
    (C::CardDelete, Full),
    (C::CardUpdateIfMatch, Partial(CHANGE_KEY)),
    (C::CardDeleteIfMatch, Partial(CHANGE_KEY)),
];

/// Google People.
const GPEOPLE: &[Row] = &[
    (C::AddressbookList, Full),
    (C::AddressbookCreate, Full),
    (C::AddressbookCreateId, Not(MINTED_ID)),
    (
        C::AddressbookUpdate,
        Partial("the Contacts system group cannot be renamed"),
    ),
    (
        C::AddressbookDelete,
        Partial("the Contacts system group cannot be deleted"),
    ),
    (
        C::AddressbookDescription,
        Not("a Google contact group carries no description"),
    ),
    (
        C::AddressbookColor,
        Not("a Google contact group carries no colour"),
    ),
    (C::CardList, Full),
    (C::CardRead, Full),
    (C::CardSearch, Not(NO_SEARCH)),
    (C::CardSearchRelevance, Not(NO_RELEVANCE)),
    (C::CardAdd, Full),
    (C::CardUpdate, Full),
    (C::CardDelete, Full),
    (C::CardUpdateIfMatch, Full),
    (
        C::CardDeleteIfMatch,
        Not("Google People cannot gate a delete on an etag"),
    ),
];

/// vdir, a directory per addressbook.
const VDIR: &[Row] = &[
    (C::AddressbookList, Full),
    (C::AddressbookCreate, Full),
    (C::AddressbookCreateId, Full),
    (C::AddressbookUpdate, Full),
    (C::AddressbookDelete, Full),
    (C::AddressbookDescription, Full),
    (C::AddressbookColor, Full),
    (C::CardList, Full),
    (C::CardRead, Full),
    (C::CardSearch, Partial(SEARCH_MATCHED)),
    (C::CardSearchRelevance, Not(NO_RELEVANCE)),
    (C::CardAdd, Full),
    (C::CardUpdate, Full),
    (C::CardDelete, Full),
    (C::CardUpdateIfMatch, Not(NO_VERSION)),
    (C::CardDeleteIfMatch, Not(NO_VERSION)),
];

/// A pimdir store a sync engine fills.
const PIMDIR: &[Row] = &[
    (C::AddressbookList, Full),
    (C::AddressbookCreate, Not(PIMDIR_COLLECTION)),
    (C::AddressbookCreateId, Not(PIMDIR_COLLECTION)),
    (C::AddressbookUpdate, Not(PIMDIR_COLLECTION)),
    (C::AddressbookDelete, Not(PIMDIR_COLLECTION)),
    (C::AddressbookDescription, Not(PIMDIR_COLLECTION)),
    (C::AddressbookColor, Not(PIMDIR_COLLECTION)),
    (C::CardList, Full),
    (C::CardRead, Full),
    (C::CardSearch, Full),
    (C::CardSearchRelevance, Full),
    (C::CardAdd, Partial(PIMDIR_QUEUED)),
    (C::CardUpdate, Partial(PIMDIR_QUEUED)),
    (C::CardDelete, Partial(PIMDIR_QUEUED)),
    (C::CardUpdateIfMatch, Full),
    (C::CardDeleteIfMatch, Full),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Every backend declares every capability, so `account capability`
    /// and the help never fall back to "not declared".
    #[test]
    fn every_backend_declares_every_capability() {
        for backend in BACKENDS {
            for capability in Capability::ALL {
                let declared = declared(backend).iter().filter(|(c, _)| *c == capability);
                assert_eq!(declared.count(), 1, "{backend} {capability}");
            }
        }
    }

    #[test]
    fn an_unserved_verb_is_refused_with_its_code_and_reason() {
        let err = require("vdir", &[C::CardUpdate, C::CardUpdateIfMatch]).unwrap_err();
        let code = pimalaya_cli::error::code_of::<ErrorCode>(&err);
        assert_eq!(code, Some(ErrorCode::CapabilityMissing));
        let message = err.to_string();
        assert!(message.contains("`card.update.if-match`"), "{message}");
        assert!(message.contains("no version to compare"), "{message}");

        assert!(require("carddav", &[C::CardUpdate, C::CardUpdateIfMatch]).is_ok());
        assert!(require("jmap", &[C::CardDelete, C::CardDeleteIfMatch]).is_ok());
        assert!(require("msgraph", &[C::CardUpdate, C::CardUpdateIfMatch]).is_ok());
        assert!(require("msgraph", &[C::AddressbookUpdate]).is_ok());
    }

    /// A JMAP account may be forbidden to create or rename an address
    /// book (`mayCreateAddressBook: false`), so neither reads `full`.
    #[test]
    fn jmap_address_book_writes_are_the_server_s() {
        for capability in [C::AddressbookCreate, C::AddressbookUpdate] {
            let support = support("jmap", capability);
            assert_eq!(support.level(), CapabilityLevel::Server, "{capability}");
        }
    }

    #[test]
    fn the_help_groups_backends_by_level() {
        let help = help("card", "delete");
        let expected = "\
Backends:
  card.delete: full on carddav, jmap, msgraph, gpeople, vdir; partial on pimdir
    pimdir: queued for the next sync, and refused when a source of the store does not support it
  card.delete.if-match: partial on carddav, jmap, msgraph; none on gpeople, vdir; full on pimdir
    carddav: sent as `If-Match`, which some servers ignore on a delete, as iCloud does
    jmap: compared with a hash of the ContactCard read just before the write, so a write landing in between is not caught
    msgraph: compared with the contact's `changeKey` read just before the write, so a write landing in between is not caught
    gpeople: Google People cannot gate a delete on an etag
    vdir: the backend exposes no version to compare, so `--if-match` is refused";
        assert_eq!(help, expected);
    }
}
