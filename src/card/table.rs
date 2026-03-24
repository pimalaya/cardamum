// This file is part of Cardamum, a CLI to manage contacts.
//
// Copyright (C) 2025 soywod <clement.douin@posteo.net>
//
// This program is free software: you can redistribute it and/or
// modify it under the terms of the GNU Affero General Public License
// as published by the Free Software Foundation, either version 3 of
// the License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public
// License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

use std::{collections::HashSet, fmt};

use comfy_table::{presets, Cell, ContentArrangement, Row, Table};
use crossterm::style::Color;
use io_addressbook::card::Card;
use serde::{ser::Serializer, Deserialize, Serialize};

use crate::table::map_color;

use super::utils::{
    build_group_member_counts, build_group_names_by_cid, contact_group_names, first_property_text,
    group_cid, group_name, is_group_card,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CardsTableMode {
    #[default]
    Contacts,
    Groups,
}

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
    cards: HashSet<Card>,
    width: Option<u16>,
    config: ListCardsTableConfig,
    mode: CardsTableMode,
}

impl CardsTable {
    pub fn with_mode(mut self, mode: CardsTableMode) -> Self {
        self.mode = mode;
        self
    }

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

    fn visible_cards(&self) -> Vec<Card> {
        let mut cards = self
            .cards
            .iter()
            .filter(|card| match self.mode {
                CardsTableMode::Contacts => !is_group_card(card),
                CardsTableMode::Groups => is_group_card(card),
            })
            .cloned()
            .collect::<Vec<_>>();

        cards.sort_by(|left, right| {
            let left_name = sort_key_name(left);
            let right_name = sort_key_name(right);
            left_name.cmp(&right_name).then(left.id.cmp(&right.id))
        });

        cards
    }
}

impl fmt::Display for CardsTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();
        table
            .load_preset(self.config.preset())
            .set_content_arrangement(ContentArrangement::DynamicFullWidth);

        match self.mode {
            CardsTableMode::Contacts => {
                let props = self.config.properties();
                let group_names_by_cid = build_group_names_by_cid(&self.cards);

                let mut headers = vec![String::from("ID"), String::from("VERSION")];
                headers.extend_from_slice(&props);
                headers.push(String::from("GROUPS"));

                table.set_header(Row::from(headers)).add_rows(
                    self.visible_cards().into_iter().map(|card| {
                        let mut row = Row::new();
                        row.max_height(1);

                        row.add_cell(Cell::new(&card.id).fg(self.config.id_color()));
                        row.add_cell(version_cell(&card, self.config.version_color()));

                        for prop in &props {
                            row.add_cell(Cell::new(
                                first_property_text(&card, prop).unwrap_or_default(),
                            ));
                        }

                        row.add_cell(Cell::new(
                            contact_group_names(&card, &group_names_by_cid).join(", "),
                        ));

                        row
                    }),
                );
            }
            CardsTableMode::Groups => {
                let member_counts_by_group_cid = build_group_member_counts(&self.cards);

                table
                    .set_header(Row::from(vec![
                        String::from("ID"),
                        String::from("VERSION"),
                        String::from("GROUP"),
                        String::from("CID"),
                        String::from("MEMBERS"),
                    ]))
                    .add_rows(self.visible_cards().into_iter().map(|card| {
                        let group_cid = group_cid(&card).unwrap_or_default();
                        let members = member_counts_by_group_cid
                            .get(&card.id)
                            .copied()
                            .unwrap_or_default();

                        let mut row = Row::new();
                        row.max_height(1);

                        row.add_cell(Cell::new(&card.id).fg(self.config.id_color()));
                        row.add_cell(version_cell(&card, self.config.version_color()));
                        row.add_cell(Cell::new(group_name(&card)));
                        row.add_cell(Cell::new(group_cid));
                        row.add_cell(Cell::new(members));

                        row
                    }));
            }
        }

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
        self.visible_cards().serialize(serializer)
    }
}

impl From<HashSet<Card>> for CardsTable {
    fn from(cards: HashSet<Card>) -> Self {
        Self {
            cards,
            width: Default::default(),
            config: Default::default(),
            mode: Default::default(),
        }
    }
}

fn version_cell(card: &Card, color: comfy_table::Color) -> Cell {
    match card.vcard.version() {
        Some(version) => Cell::new(version).fg(color),
        None => Cell::new(""),
    }
}

fn sort_key_name(card: &Card) -> String {
    group_name(card)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_card(id: &str, addressbook_id: &str, content: &str) -> Card {
        Card {
            id: id.to_string(),
            addressbook_id: addressbook_id.to_string(),
            vcard: Card::parse(content).unwrap(),
        }
    }

    #[test]
    fn contacts_mode_hides_group_cards_and_shows_group_names() {
        let contact = parse_card(
            "123",
            "contacts",
            "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Alice\r\nEMAIL:alice@example.com\r\nGrouping:42\r\nEND:VCARD\r\n",
        );
        let group = parse_card(
            "MacGroup-42",
            "contacts",
            "BEGIN:VCARD\r\nVERSION:3.0\r\nN:导师组\r\nUID:MacGroup-42\r\nCID:42\r\nX-TYPE:GROUP\r\nX-ADDRESSBOOKSERVER-KIND:group\r\nEND:VCARD\r\n",
        );

        let table = CardsTable::from(HashSet::from([contact, group]));
        let rendered = table.to_string();

        assert!(rendered.contains("Alice"));
        assert!(rendered.contains("导师组"));
        assert!(!rendered.contains("MacGroup-42"));
    }

    #[test]
    fn groups_mode_lists_groups_with_member_count() {
        let contact = parse_card(
            "123",
            "contacts",
            "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Alice\r\nGrouping:42\r\nEND:VCARD\r\n",
        );
        let group = parse_card(
            "MacGroup-42",
            "contacts",
            "BEGIN:VCARD\r\nVERSION:3.0\r\nN:导师组\r\nUID:MacGroup-42\r\nCID:42\r\nX-TYPE:GROUP\r\nX-ADDRESSBOOKSERVER-KIND:group\r\nEND:VCARD\r\n",
        );

        let cards = HashSet::from([contact, group]);
        let table = CardsTable::from(cards.clone()).with_mode(CardsTableMode::Groups);
        let rendered = table.to_string();

        assert!(rendered.contains("导师组"));
        assert!(!rendered.contains("Alice"));
        assert_eq!(
            build_group_member_counts(&cards).get("MacGroup-42"),
            Some(&1)
        );
    }
}
