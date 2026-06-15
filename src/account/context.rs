//! Merged runtime account: the DTO every command consumes.
//!
//! Built by the dispatch layer in this order:
//!
//! 1. [`Account::default`] (all fields `None`).
//! 2. Fold the global [`Config`] via `Account::from(config)`.
//! 3. Fold the selected `[accounts.<name>]` via [`Account::merge`]
//!    with `Account::from(account_config)`.

use anyhow::{Result, bail};
use comfy_table::{Color as TableColor, ContentArrangement, presets};
use crossterm::style::Color;

use crate::config::{
    AccountConfig, AddressbookListTableConfig, CardListTableConfig, Config, TableArrangementConfig,
};

const DEFAULT_CARDS_LIST_PAGE_SIZE: u32 = 25;

#[derive(Debug, Default)]
pub struct Account {
    pub table_preset: Option<String>,
    pub table_arrangement: Option<TableArrangementConfig>,

    pub cards_list_page_size: Option<u32>,

    /// Fallback addressbook id for `card` commands when their
    /// `-k/--addressbook` flag is omitted.
    pub addressbook_default: Option<String>,

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

            addressbooks_list_table: merge_addressbook_table(
                self.addressbooks_list_table,
                other.addressbooks_list_table,
            ),
            cards_list_table: merge_card_table(self.cards_list_table, other.cards_list_table),
        }
    }

    /// Effective `comfy_table` preset string. Defaults to
    /// `UTF8_FULL_CONDENSED`.
    pub fn table_preset(&self) -> &str {
        self.table_preset
            .as_deref()
            .unwrap_or(presets::UTF8_FULL_CONDENSED)
    }

    /// Effective `comfy_table` content arrangement. Defaults to
    /// `Dynamic`.
    #[allow(dead_code)]
    pub fn table_arrangement(&self) -> ContentArrangement {
        self.table_arrangement
            .clone()
            .unwrap_or(TableArrangementConfig::Dynamic)
            .into()
    }

    /// Effective default page size for `cards list` when the
    /// `-s/--page-size` flag is not passed. Defaults to 25.
    pub fn cards_list_page_size(&self) -> u32 {
        self.cards_list_page_size
            .unwrap_or(DEFAULT_CARDS_LIST_PAGE_SIZE)
    }

    /// Resolves the addressbook id a shared-API command operates on: the
    /// `-k/--addressbook` flag wins; otherwise the `addressbook.default`
    /// config is used; otherwise the command bails.
    pub fn addressbook_id(&self, flag: Option<String>) -> Result<String> {
        if let Some(id) = flag.or_else(|| self.addressbook_default.clone()) {
            return Ok(id);
        }

        bail!("Missing addressbook id; pass -k/--addressbook or set addressbook.default")
    }

    // ── addressbooks list — column colors ───────────────────────────
    pub fn addressbooks_list_table_id_color(&self) -> TableColor {
        map_color_or(self.addressbooks_list_table.id_color, Color::Red)
    }
    pub fn addressbooks_list_table_name_color(&self) -> TableColor {
        map_color_or(self.addressbooks_list_table.name_color, Color::Green)
    }
    pub fn addressbooks_list_table_description_color(&self) -> TableColor {
        map_color_or(self.addressbooks_list_table.description_color, Color::Reset)
    }
    pub fn addressbooks_list_table_color_color(&self) -> TableColor {
        map_color_or(self.addressbooks_list_table.color_color, Color::Reset)
    }

    // ── cards list — column colors ──────────────────────────────────
    pub fn cards_list_table_id_color(&self) -> TableColor {
        map_color_or(self.cards_list_table.id_color, Color::Red)
    }
    pub fn cards_list_table_fn_color(&self) -> TableColor {
        map_color_or(self.cards_list_table.fn_color, Color::Green)
    }
    pub fn cards_list_table_email_color(&self) -> TableColor {
        map_color_or(self.cards_list_table.email_color, Color::Blue)
    }
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
            addressbooks_list_table: config.addressbook.list.table,
            cards_list_table: config.card.list.table,
        }
    }
}

/// Maps a [`crossterm::style::Color`] (deserialized from TOML) into a
/// [`comfy_table::Color`] used by the renderers, substituting
/// `fallback` when the TOML field is unset.
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
