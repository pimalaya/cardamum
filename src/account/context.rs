//! # Account context
//!
//! The merged runtime account every command consumes.
//!
//! The dispatch layer builds it by folding the global [`Config`] onto a
//! defaulted account, then the selected `[accounts.<name>]` block onto
//! that, so the narrower value always wins.

use anyhow::{Result, bail};
use crossterm::style::Color;
use pimalaya_cli::table::{Color as TableColor, ContentArrangement};
use pimalaya_config::command::CommandConfig;

use crate::{
    config::{
        AccountConfig, AddressbookListTableConfig, CardListTableConfig, Config,
        TableArrangementConfig,
    },
    shared::table::DEFAULT_PRESET,
};

/// Page size a `card list` falls back to when nothing sets one.
const DEFAULT_CARDS_LIST_PAGE_SIZE: u32 = 25;

/// The configuration values a command resolves its defaults from.
#[derive(Debug, Default)]
pub struct Account {
    /// `comfy_table` preset string the tables are drawn with.
    pub table_preset: Option<String>,
    /// How the tables arrange their content in the available width.
    pub table_arrangement: Option<TableArrangementConfig>,
    /// Default page size of `card list`.
    pub cards_list_page_size: Option<u32>,
    /// Fallback addressbook id for `card` commands when their
    /// `-k/--addressbook` flag is omitted.
    pub addressbook_default: Option<String>,
    /// Command a card is edited through, spawned on the path of a
    /// temporary vCard file.
    pub card_composer: Option<CommandConfig>,
    /// Per-column color overrides for `addressbooks list`.
    pub addressbooks_list_table: AddressbookListTableConfig,
    /// Per-column color overrides for `cards list`.
    pub cards_list_table: CardListTableConfig,
}

impl Account {
    /// Folds `other`'s set fields on top of `self`.
    pub fn merge(self, other: Self) -> Self {
        Self {
            table_preset: other.table_preset.or(self.table_preset),
            table_arrangement: other.table_arrangement.or(self.table_arrangement),

            cards_list_page_size: other.cards_list_page_size.or(self.cards_list_page_size),

            addressbook_default: other.addressbook_default.or(self.addressbook_default),
            card_composer: other.card_composer.or(self.card_composer),

            addressbooks_list_table: merge_addressbook_table(
                self.addressbooks_list_table,
                other.addressbooks_list_table,
            ),
            cards_list_table: merge_card_table(self.cards_list_table, other.cards_list_table),
        }
    }

    /// Effective preset string, falling back to full condensed borders.
    pub fn table_preset(&self) -> &str {
        self.table_preset.as_deref().unwrap_or(DEFAULT_PRESET)
    }

    /// Effective content arrangement, falling back to dynamic.
    #[allow(dead_code)]
    pub fn table_arrangement(&self) -> ContentArrangement {
        self.table_arrangement
            .clone()
            .unwrap_or(TableArrangementConfig::Dynamic)
            .into()
    }

    /// Effective page size of `card list`, falling back to 25.
    pub fn cards_list_page_size(&self) -> u32 {
        self.cards_list_page_size
            .unwrap_or(DEFAULT_CARDS_LIST_PAGE_SIZE)
    }

    /// Resolves the addressbook id a command operates on.
    ///
    /// The flag wins, then the `addressbook.default` config, otherwise
    /// this bails. An empty id bails too: a backend builds a path from
    /// it, where an empty segment addresses the parent collection.
    pub fn addressbook_id(&self, flag: Option<String>) -> Result<String> {
        let Some(id) = flag.or_else(|| self.addressbook_default.clone()) else {
            bail!("Missing addressbook id; pass -k/--addressbook or set addressbook.default")
        };

        if id.trim().is_empty() {
            bail!("Addressbook id cannot be empty");
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

/// Effective column colors of the `addressbook list` table, each falling
/// back to its built-in default when the configuration leaves it unset.
impl Account {
    /// Color of the addressbook id column. Defaults to red.
    pub fn addressbooks_list_table_id_color(&self) -> TableColor {
        map_color_or(self.addressbooks_list_table.id_color, Color::Red)
    }
    /// Color of the addressbook name column. Defaults to green.
    pub fn addressbooks_list_table_name_color(&self) -> TableColor {
        map_color_or(self.addressbooks_list_table.name_color, Color::Green)
    }
    /// Color of the addressbook description column. Defaults to reset.
    pub fn addressbooks_list_table_description_color(&self) -> TableColor {
        map_color_or(self.addressbooks_list_table.description_color, Color::Reset)
    }
    /// Color of the addressbook color-marker column. Defaults to reset.
    pub fn addressbooks_list_table_color_color(&self) -> TableColor {
        map_color_or(self.addressbooks_list_table.color_color, Color::Reset)
    }
}

/// Effective column colors of the `card list` table, each falling back to
/// its built-in default when the configuration leaves it unset.
impl Account {
    /// Color of the card id column. Defaults to red.
    pub fn cards_list_table_id_color(&self) -> TableColor {
        map_color_or(self.cards_list_table.id_color, Color::Red)
    }
    /// Color of the card display-name (`FN`) column. Defaults to green.
    pub fn cards_list_table_fn_color(&self) -> TableColor {
        map_color_or(self.cards_list_table.fn_color, Color::Green)
    }
    /// Color of the card email column. Defaults to blue.
    pub fn cards_list_table_email_color(&self) -> TableColor {
        map_color_or(self.cards_list_table.email_color, Color::Blue)
    }
    /// Color of the card telephone column. Defaults to dark yellow.
    pub fn cards_list_table_tel_color(&self) -> TableColor {
        map_color_or(self.cards_list_table.tel_color, Color::DarkYellow)
    }
}

impl From<Config> for Account {
    fn from(config: Config) -> Self {
        Self {
            table_preset: config.table.preset,
            table_arrangement: config.table.arrangement,
            cards_list_page_size: config.card.list.page_size,
            addressbook_default: config.addressbook.default,
            card_composer: config.card.composer,
            addressbooks_list_table: config.addressbook.list.table,
            cards_list_table: config.card.list.table,
        }
    }
}

impl From<AccountConfig> for Account {
    fn from(config: AccountConfig) -> Self {
        Self {
            table_preset: config.table.preset,
            table_arrangement: config.table.arrangement,
            cards_list_page_size: config.card.list.page_size,
            addressbook_default: config.addressbook.default,
            card_composer: config.card.composer,
            addressbooks_list_table: config.addressbook.list.table,
            cards_list_table: config.card.list.table,
        }
    }
}

/// Maps a configured [`Color`] onto the one the renderers use.
///
/// Substitutes `fallback` when the configuration leaves the field unset.
pub(crate) fn map_color_or(color: Option<Color>, fallback: Color) -> TableColor {
    match color.unwrap_or(fallback) {
        Color::Reset => TableColor::Reset,
        Color::Black => TableColor::Black,
        Color::DarkGrey => TableColor::DarkGrey,
        Color::Red => TableColor::Red,
        Color::DarkRed => TableColor::DarkRed,
        Color::Green => TableColor::Green,
        Color::DarkGreen => TableColor::DarkGreen,
        Color::Yellow => TableColor::Yellow,
        Color::DarkYellow => TableColor::DarkYellow,
        Color::Blue => TableColor::Blue,
        Color::DarkBlue => TableColor::DarkBlue,
        Color::Magenta => TableColor::Magenta,
        Color::DarkMagenta => TableColor::DarkMagenta,
        Color::Cyan => TableColor::Cyan,
        Color::DarkCyan => TableColor::DarkCyan,
        Color::White => TableColor::White,
        Color::Grey => TableColor::Grey,
        Color::Rgb { r, g, b } => TableColor::Rgb { r, g, b },
        Color::AnsiValue(n) => TableColor::AnsiValue(n),
    }
}

fn merge_addressbook_table(
    base: AddressbookListTableConfig,
    over: AddressbookListTableConfig,
) -> AddressbookListTableConfig {
    AddressbookListTableConfig {
        id_color: over.id_color.or(base.id_color),
        name_color: over.name_color.or(base.name_color),
        description_color: over.description_color.or(base.description_color),
        color_color: over.color_color.or(base.color_color),
    }
}

fn merge_card_table(base: CardListTableConfig, over: CardListTableConfig) -> CardListTableConfig {
    CardListTableConfig {
        id_color: over.id_color.or(base.id_color),
        fn_color: over.fn_color.or(base.fn_color),
        email_color: over.email_color.or(base.email_color),
        tel_color: over.tel_color.or(base.tel_color),
    }
}
