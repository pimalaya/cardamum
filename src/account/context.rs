//! # Account context
//!
//! The merged runtime account every command consumes.
//!
//! The dispatch layer builds it by folding the global [`Config`] onto a
//! defaulted account, then the selected `[accounts.<name>]` block onto
//! that, so the narrower value always wins.

use anyhow::{Result, bail};
use pimalaya_cli::error::CodedError;
use pimalaya_config::command::CommandConfig;

use crate::{
    config::{AccountConfig, Config},
    error::ErrorCode,
};

/// Page size a `card list` falls back to when nothing sets one.
const DEFAULT_CARDS_LIST_PAGE_SIZE: u32 = 25;

/// The configuration values a command resolves its defaults from.
#[derive(Debug, Default)]
pub struct Account {
    /// Default page size of `card list`.
    pub cards_list_page_size: Option<u32>,
    /// Fallback addressbook id for `card` commands when their
    /// `-k/--addressbook` flag is omitted.
    pub addressbook_default: Option<String>,
    /// Command a card is edited through, spawned on the path of a
    /// temporary vCard file.
    pub card_composer: Option<CommandConfig>,
}

impl Account {
    /// Folds `other`'s set fields on top of `self`.
    pub fn merge(self, other: Self) -> Self {
        Self {
            cards_list_page_size: other.cards_list_page_size.or(self.cards_list_page_size),
            addressbook_default: other.addressbook_default.or(self.addressbook_default),
            card_composer: other.card_composer.or(self.card_composer),
        }
    }

    /// Effective page size of `card list`, falling back to 25.
    pub fn cards_list_page_size(&self) -> u32 {
        self.cards_list_page_size
            .unwrap_or(DEFAULT_CARDS_LIST_PAGE_SIZE)
    }

    /// Resolves the addressbook id a command operates on.
    ///
    /// The flag wins, then the `addressbook.default` config, otherwise
    /// this bails. An empty id bails too, coded `addressbook-not-found` as
    /// it names none: a backend builds a path from it, where an empty
    /// segment addresses the parent collection.
    pub fn addressbook_id(&self, flag: Option<String>) -> Result<String> {
        let Some(id) = flag.or_else(|| self.addressbook_default.clone()) else {
            bail!("Missing addressbook id; pass -k/--addressbook or set addressbook.default")
        };

        if id.trim().is_empty() {
            let message = "Addressbook id cannot be empty";
            return Err(CodedError::new(ErrorCode::AddressbookNotFound, message).into());
        }

        Ok(id)
    }

    /// Resolves the composer a card is edited through.
    ///
    /// The flag wins, taken as a shell line, then the `card.composer`
    /// config, otherwise this bails naming both ways of setting one.
    pub fn card_composer(&self, flag: Option<String>) -> Result<CommandConfig> {
        if let Some(line) = flag {
            return Ok(CommandConfig::Shell(line));
        }

        let Some(composer) = self.card_composer.clone() else {
            bail!(
                "No composer configured; set card.composer or pass --composer <COMMAND>, \
                 which is spawned on the path of the vCard to edit"
            )
        };

        Ok(composer)
    }
}

impl From<Config> for Account {
    fn from(config: Config) -> Self {
        Self {
            cards_list_page_size: config.card.list.page_size,
            addressbook_default: config.addressbook.default,
            card_composer: config.card.composer,
        }
    }
}

impl From<AccountConfig> for Account {
    fn from(config: AccountConfig) -> Self {
        Self {
            cards_list_page_size: config.card.list.page_size,
            addressbook_default: config.addressbook.default,
            card_composer: config.card.composer,
        }
    }
}
