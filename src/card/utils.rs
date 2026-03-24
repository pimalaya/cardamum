use std::collections::{HashMap, HashSet};

use io_addressbook::card::{Card, VCardValue};

pub(crate) fn is_group_card(card: &Card) -> bool {
    property_texts(card, "X-ADDRESSBOOKSERVER-KIND")
        .into_iter()
        .any(|value| value.eq_ignore_ascii_case("group"))
        || property_texts(card, "X-TYPE")
            .into_iter()
            .any(|value| value.eq_ignore_ascii_case("group"))
        || property_texts(card, "KIND")
            .into_iter()
            .any(|value| value.eq_ignore_ascii_case("group"))
        || card
            .vcard
            .uid()
            .is_some_and(|uid| uid.starts_with("MacGroup-"))
}

pub(crate) fn first_property_text(card: &Card, name: &str) -> Option<String> {
    property_texts(card, name).into_iter().next()
}

pub(crate) fn property_texts(card: &Card, name: &str) -> Vec<String> {
    card.entries()
        .filter(|entry| entry.name.as_str().eq_ignore_ascii_case(name))
        .flat_map(|entry| entry.values.iter())
        .filter_map(|value| match value {
            VCardValue::Text(text) if !text.is_empty() => Some(text.clone()),
            _ => None,
        })
        .collect()
}

pub(crate) fn group_name(card: &Card) -> String {
    first_property_text(card, "FN")
        .or_else(|| first_property_text(card, "N"))
        .unwrap_or_else(|| card.id.clone())
}

pub(crate) fn group_cid(card: &Card) -> Option<String> {
    first_property_text(card, "CID").or_else(|| {
        card.vcard
            .uid()
            .and_then(|uid| uid.strip_prefix("MacGroup-"))
            .map(str::to_owned)
    })
}

pub(crate) fn group_lookup_keys(card: &Card) -> Vec<String> {
    let mut keys = Vec::new();

    keys.push(card.id.clone());

    if let Some(id) = card.id.strip_prefix("MacGroup-") {
        keys.push(id.to_owned());
    }

    if let Some(uid) = card.vcard.uid() {
        keys.push(uid.to_owned());

        if let Some(id) = uid.strip_prefix("MacGroup-") {
            keys.push(id.to_owned());
        }
    }

    if let Some(cid) = group_cid(card) {
        keys.push(cid);
    }

    keys.sort();
    keys.dedup();
    keys
}

pub(crate) fn build_group_names_by_cid(cards: &HashSet<Card>) -> HashMap<String, String> {
    let mut names = HashMap::new();

    for card in cards.iter().filter(|card| is_group_card(card)) {
        let name = group_name(card);

        for key in group_lookup_keys(card) {
            names.insert(key, name.clone());
        }
    }

    names
}

pub(crate) fn build_group_member_counts(cards: &HashSet<Card>) -> HashMap<String, usize> {
    let mut counts_by_grouping = HashMap::new();

    for card in cards.iter().filter(|card| !is_group_card(card)) {
        let groupings = property_texts(card, "Grouping");

        for grouping in groupings {
            *counts_by_grouping.entry(grouping).or_insert(0) += 1;
        }
    }

    let mut counts = HashMap::new();

    for card in cards.iter().filter(|card| is_group_card(card)) {
        let mut total = 0usize;

        for key in group_lookup_keys(card) {
            total += counts_by_grouping.get(&key).copied().unwrap_or_default();
        }

        counts.insert(card.id.clone(), total);
    }

    counts
}

pub(crate) fn contact_group_names(
    card: &Card,
    group_names_by_cid: &HashMap<String, String>,
) -> Vec<String> {
    let mut names = property_texts(card, "Grouping")
        .into_iter()
        .map(|grouping| {
            group_names_by_cid
                .get(&grouping)
                .cloned()
                .unwrap_or(grouping)
        })
        .collect::<Vec<_>>();

    names.sort();
    names.dedup();
    names
}
