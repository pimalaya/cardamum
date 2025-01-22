use std::fmt;

use addressbook::Cards;
use comfy_table::{presets, Cell, ContentArrangement, Row, Table};
use crossterm::style::Color;
use serde::{ser::Serializer, Deserialize, Serialize};

use crate::table::map_color;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ListCardsTableConfig {
    pub preset: Option<String>,
    pub id_color: Option<Color>,
    pub version_color: Option<Color>,
    pub properties: Option<Vec<String>>,
}

impl ListCardsTableConfig {
    pub fn preset(&self) -> &str {
        self.preset.as_deref().unwrap_or(presets::ASCII_MARKDOWN)
    }

    pub fn id_color(&self) -> comfy_table::Color {
        map_color(self.id_color.unwrap_or(Color::Red))
    }

    pub fn version_color(&self) -> comfy_table::Color {
        map_color(self.version_color.unwrap_or(Color::Green))
    }

    pub fn properties(&self) -> Vec<String> {
        self.properties.clone().unwrap_or(vec![
            String::from("FN"),
            String::from("EMAIL"),
            String::from("TEL"),
        ])
    }
}

pub struct CardsTable {
    cards: Cards,
    width: Option<u16>,
    config: ListCardsTableConfig,
}

impl CardsTable {
    pub fn with_some_width(mut self, width: Option<u16>) -> Self {
        self.width = width;
        self
    }

    pub fn with_some_preset(mut self, preset: Option<String>) -> Self {
        self.config.preset = preset;
        self
    }

    pub fn with_some_id_color(mut self, color: Option<Color>) -> Self {
        self.config.id_color = color;
        self
    }

    pub fn with_some_version_color(mut self, color: Option<Color>) -> Self {
        self.config.version_color = color;
        self
    }
}

impl From<Cards> for CardsTable {
    fn from(cards: Cards) -> Self {
        Self {
            cards,
            width: None,
            config: Default::default(),
        }
    }
}

impl fmt::Display for CardsTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        let props = self.config.properties();

        let mut headers = vec![String::from("ID"), String::from("VERSION")];

        headers.extend_from_slice(&props);

        table
            .load_preset(self.config.preset())
            .set_content_arrangement(ContentArrangement::DynamicFullWidth)
            .set_header(Row::from(headers))
            .add_rows(self.cards.iter().map(|card| {
                let mut row = Row::new();
                row.max_height(1);

                row.add_cell(Cell::new(&card.id).fg(self.config.id_color()));
                row.add_cell(Cell::new(&card.version).fg(self.config.version_color()));

                for prop in &props {
                    if let Some(prop) = card.properties.get(prop) {
                        row.add_cell(Cell::new(prop));
                    }
                }

                row
            }));

        if let Some(width) = self.width {
            table.set_width(width);
        }

        writeln!(f)?;
        write!(f, "{table}")?;
        writeln!(f)?;
        Ok(())
    }
}

impl Serialize for CardsTable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.cards.serialize(serializer)
    }
}
