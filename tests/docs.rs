//! # README tests
//!
//! The README's budgets of the guidelines: the Features bullets and the
//! section order (readme-008, readme-007), and its terminal block printing
//! what it shows (readme-006).

use std::fs;
#[cfg(feature = "vdir")]
use std::process::{Command, Stdio};

#[cfg(feature = "vdir")]
use serde_json::Value;
#[cfg(feature = "vdir")]
use tempfile::TempDir;

/// The README, read from the repository root.
fn readme() -> String {
    fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md")).unwrap()
}

/// The README's `## ` sections, in order.
fn readme_sections(readme: &str) -> Vec<&str> {
    readme
        .lines()
        .filter_map(|line| line.strip_prefix("## "))
        .collect()
}

#[test]
fn the_readme_features_are_eight_short_bullets_at_most() {
    let readme = readme();
    let features: Vec<&str> = readme
        .split("\n## Features\n")
        .nth(1)
        .expect("a Features section")
        .split("\n## ")
        .next()
        .unwrap()
        .lines()
        .filter(|line| line.starts_with("- "))
        .collect();

    assert!(!features.is_empty() && features.len() <= 8, "{features:#?}");
    for feature in features {
        assert!(feature.chars().count() <= 120, "{feature}");
    }
}

#[test]
fn the_readme_sections_follow_the_cli_order() {
    assert_eq!(
        readme_sections(&readme()),
        [
            "Table of contents",
            "Features",
            "RFC coverage",
            "Installation",
            "Configuration",
            "Usage",
            "Alternatives",
            "License",
            "Social",
            "Sponsoring",
        ]
    );
}

/// The two cards of the vdir addressbook the README's header block lists,
/// by file name.
#[cfg(feature = "vdir")]
const README_VDIR: [(&str, &str); 2] = [
    (
        "ada.vcf",
        "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:ada\r\nFN:Ada Lovelace\r\nN:Lovelace;Ada;;;\r\nEMAIL:ada@example.org\r\nTEL:+44 20 7946 0000\r\nEND:VCARD\r\n",
    ),
    (
        "alan.vcf",
        "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:alan\r\nFN:Alan Turing\r\nN:Turing;Alan;;;\r\nEMAIL:alan@example.org\r\nEND:VCARD\r\n",
    ),
];

/// The content of the first fenced block of `lang` in `text`, and the
/// rest of `text` after it.
#[cfg(feature = "vdir")]
fn fenced<'a>(text: &'a str, lang: &str) -> (&'a str, &'a str) {
    let open = format!("```{lang}\n");
    let start = text.find(&open).expect("a fenced block") + open.len();
    let end = start + text[start..].find("\n```").expect("a closing fence");
    (&text[start..end], &text[end + 4..])
}

/// The command line under the description, run over a vdir addressbook
/// holding two cards, prints the JSON shown after it.
#[cfg(feature = "vdir")]
#[test]
fn the_readme_header_block_prints_what_it_shows() {
    let readme = readme();
    let (command, rest) = fenced(&readme, "sh");
    let (shown, _) = fenced(rest, "json");
    let args: Vec<&str> = command
        .strip_prefix("$ cardamum ")
        .expect("a cardamum command line")
        .split_whitespace()
        .collect();

    let dir = TempDir::new().unwrap();
    let book = dir.path().join("vdir").join("personal");
    fs::create_dir_all(&book).unwrap();
    for (name, card) in README_VDIR {
        fs::write(book.join(name), card).unwrap();
    }

    let config = dir.path().join("config.toml");
    let toml = format!(
        "[accounts.home]\ndefault = true\nvdir.root = {:?}\naddressbook.default = \"personal\"\n",
        dir.path().join("vdir").display().to_string(),
    );
    fs::write(&config, toml).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_cardamum"))
        .arg("-c")
        .arg(&config)
        .args(&args)
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", dir.path())
        .env_remove("XDG_CONFIG_DIRS")
        .stdin(Stdio::null())
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    let printed: Value = serde_json::from_slice(&output.stdout).unwrap();
    let shown: Value = serde_json::from_str(shown).unwrap();
    assert_eq!(printed, shown);
}
