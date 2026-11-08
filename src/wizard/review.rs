//! # Review
//!
//! Shows the pick in plain words through pimalaya-cli's review, one
//! `Contacts` row, and lets the user change it: back to Auto, to another
//! option discovery found, or to a server typed by hand (see
//! [`crate::wizard::server`]). Protocol jargon stays behind Change.

use std::fmt;

use anyhow::Result;
use pimalaya_cli::{
    prompt,
    wizard::review::{self, ReviewAction, ReviewRow},
};

use crate::wizard::{search::Discovered, server};

/// Shows the review of `pick` until the user continues, answering the
/// pick to sign in with.
///
/// `found` holds the ranked options discovery found, Auto first, empty
/// when nothing was.
pub fn prompt(found: &[Discovered], mut pick: Discovered, ortie: bool) -> Result<Discovered> {
    let custom = cfg!(any(feature = "carddav", feature = "jmap"));

    loop {
        let row = ReviewRow {
            name: String::from("Contacts"),
            line: pick.sentence(ortie),
        };

        if review::prompt(&[row])? == ReviewAction::Continue {
            return Ok(pick);
        }

        let items = change_items(found, ortie, custom);

        match prompt::item("Contacts:", items, None)?.target {
            ChangeTarget::Found(index) => pick = found[index].clone(),
            ChangeTarget::Custom => {
                if let Some(typed) = server::prompt()? {
                    pick = typed;
                }
            }
        }
    }
}

/// One entry of the Change list.
#[derive(Debug, Eq, PartialEq)]
struct ChangeItem {
    label: String,
    target: ChangeTarget,
}

/// What a Change entry picks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ChangeTarget {
    /// The option found at this index, Auto being the first.
    Found(usize),
    /// A server typed by hand.
    Custom,
}

impl fmt::Display for ChangeItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label)
    }
}

/// The Change list: Auto naming what it is, every option found, then the
/// custom server when a protocol taking one is compiled.
fn change_items(found: &[Discovered], ortie: bool, custom: bool) -> Vec<ChangeItem> {
    let mut items = Vec::new();

    if let Some(auto) = found.first() {
        items.push(ChangeItem {
            label: format!("Auto ({})", auto.sentence(ortie)),
            target: ChangeTarget::Found(0),
        });
    }

    for (index, label) in option_labels(found).into_iter().enumerate() {
        items.push(ChangeItem {
            label,
            target: ChangeTarget::Found(index),
        });
    }

    if custom {
        items.push(ChangeItem {
            label: String::from("Custom server URL"),
            target: ChangeTarget::Custom,
        });
    }

    items
}

/// The Change labels of `found`, as `CardDAV · Password`, naming the
/// server's host only where two labels would read the same, and its whole
/// URL where the hosts do too.
fn option_labels(found: &[Discovered]) -> Vec<String> {
    let short: Vec<String> = found
        .iter()
        .map(|option| format!("{} · {}", option.kind.protocol(), option.sign_in))
        .collect();

    let host: Vec<String> = found
        .iter()
        .zip(&short)
        .map(|(option, label)| match option.kind.host() {
            Some(host) => format!("{label} · {host}"),
            None => label.clone(),
        })
        .collect();

    let unique = |labels: &[String], label: &String| {
        labels.iter().filter(|other| *other == label).count() == 1
    };

    found
        .iter()
        .enumerate()
        .map(|(index, option)| {
            if unique(&short, &short[index]) {
                return short[index].clone();
            }

            if unique(&host, &host[index]) {
                return host[index].clone();
            }

            match option.kind.server() {
                Some(url) => format!("{} · {url}", short[index]),
                None => short[index].clone(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use url::Url;

    use crate::wizard::{
        review::*,
        search::{DiscoveredKind, SignIn},
    };

    fn option(kind: DiscoveredKind, sign_in: SignIn) -> Discovered {
        Discovered {
            kind,
            sign_in,
            username: None,
        }
    }

    fn carddav(url: &str) -> DiscoveredKind {
        DiscoveredKind::Carddav(Url::parse(url).unwrap())
    }

    fn labels(items: &[ChangeItem]) -> Vec<&str> {
        items.iter().map(|item| item.label.as_str()).collect()
    }

    #[test]
    fn a_change_names_the_server_only_where_two_read_the_same() {
        let found = [
            option(carddav("https://dav.example.org/"), SignIn::Password),
            option(carddav("https://carddav.example.org/"), SignIn::Password),
            option(carddav("https://carddav.example.org/"), SignIn::Token),
            option(
                DiscoveredKind::Jmap(Url::parse("https://api.example.org/jmap").unwrap()),
                SignIn::Password,
            ),
            option(carddav("https://shared.example.org/alice/"), SignIn::Oauth),
            option(carddav("https://shared.example.org/bob/"), SignIn::Oauth),
            option(DiscoveredKind::Gpeople, SignIn::Oauth),
        ];

        assert_eq!(
            option_labels(&found),
            [
                "CardDAV · Password · dav.example.org",
                "CardDAV · Password · carddav.example.org",
                "CardDAV · API token",
                "JMAP · Password",
                "CardDAV · OAuth 2.0 · https://shared.example.org/alice/",
                "CardDAV · OAuth 2.0 · https://shared.example.org/bob/",
                "People API · OAuth 2.0",
            ]
        );
    }

    #[test]
    fn a_change_lists_auto_then_each_option_then_a_custom_server() {
        let found = [
            option(carddav("https://dav.example.org/"), SignIn::Password),
            option(carddav("https://dav.example.org/"), SignIn::Token),
        ];

        let items = change_items(&found, true, true);

        assert_eq!(
            labels(&items),
            [
                "Auto (CardDAV at dav.example.org, with your password)",
                "CardDAV · Password",
                "CardDAV · API token",
                "Custom server URL",
            ]
        );
        assert_eq!(items[0].target, ChangeTarget::Found(0));
        assert_eq!(items[2].target, ChangeTarget::Found(1));
        assert_eq!(items[3].target, ChangeTarget::Custom);

        assert_eq!(
            labels(&change_items(&[], true, true)),
            ["Custom server URL"]
        );
        assert_eq!(
            labels(&change_items(&found[..1], true, false)),
            [
                "Auto (CardDAV at dav.example.org, with your password)",
                "CardDAV · Password",
            ]
        );
    }
}
