use std::{borrow::Cow, fmt};

use addressbook::Addressbooks;
use comfy_table::{presets, Cell, ContentArrangement, Row, Table};
use crossterm::style::Color;
use serde::{
    de::{value::CowStrDeserializer, IntoDeserializer},
    Deserialize, Serialize, Serializer,
};

use crate::table::map_color;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ListAddressbooksTableConfig {
    pub preset: Option<String>,

    pub id_color: Option<Color>,
    pub name_color: Option<Color>,
    pub desc_color: Option<Color>,
}

impl ListAddressbooksTableConfig {
    pub fn preset(&self) -> &str {
        self.preset.as_deref().unwrap_or(presets::ASCII_MARKDOWN)
    }

    pub fn id_color(&self) -> comfy_table::Color {
        map_color(self.id_color.unwrap_or(Color::Red))
    }

    pub fn name_color(&self) -> comfy_table::Color {
        map_color(self.name_color.unwrap_or(Color::Reset))
    }

    pub fn desc_color(&self) -> comfy_table::Color {
        map_color(self.name_color.unwrap_or(Color::Green))
    }
}

pub struct AddressbooksTable {
    addressbooks: Addressbooks,
    width: Option<u16>,
    config: ListAddressbooksTableConfig,
}

impl AddressbooksTable {
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

    pub fn with_some_name_color(mut self, color: Option<Color>) -> Self {
        self.config.name_color = color;
        self
    }

    pub fn with_some_desc_color(mut self, color: Option<Color>) -> Self {
        self.config.desc_color = color;
        self
    }
}

impl From<Addressbooks> for AddressbooksTable {
    fn from(addressbooks: Addressbooks) -> Self {
        Self {
            addressbooks,
            width: None,
            config: Default::default(),
        }
    }
}

impl fmt::Display for AddressbooksTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_preset(self.config.preset())
            .set_content_arrangement(ContentArrangement::DynamicFullWidth)
            .set_header(Row::from([
                Cell::new("ID"),
                Cell::new("NAME"),
                Cell::new("DESC"),
                Cell::new("COLOR"),
            ]))
            .add_rows(self.addressbooks.iter().map(|addressbook| {
                let mut row = Row::new();
                row.max_height(1);

                row.add_cell(Cell::new(&addressbook.id).fg(self.config.id_color()));
                row.add_cell(Cell::new(&addressbook.name).fg(self.config.name_color()));

                if let Some(desc) = &addressbook.desc {
                    row.add_cell(Cell::new(desc).fg(self.config.desc_color()));
                } else {
                    row.add_cell(Cell::new(String::new()));
                }

                let mut color_cell = Cell::new("");

                if let Some(color) = &addressbook.color {
                    color_cell = Cell::new(color);

                    // hash tag (1) + rgb hex code (2 + 2 + 2)
                    if color.len() >= 7 {
                        let deserializer: CowStrDeserializer<serde::de::value::Error> =
                            Cow::from(unsafe { color.get_unchecked(..7) }).into_deserializer();

                        if let Ok(rgb) = Color::deserialize(deserializer) {
                            color_cell = color_cell.bg(map_color(rgb));
                        };
                    }
                }

                row.add_cell(color_cell);

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

impl Serialize for AddressbooksTable {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.addressbooks.serialize(serializer)
    }
}
