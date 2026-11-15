//! # Listing errors
//!
//! The built binary's `card list` over a vdir address book holding a card
//! it cannot read: the other cards list, an `errors` entry names it, and
//! the command exits 3 once it printed its result.

#![cfg(all(unix, feature = "vdir"))]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Output, Stdio},
};

use serde_json::Value;
use tempfile::TempDir;

fn card(name: &str) -> String {
    format!("BEGIN:VCARD\r\nVERSION:4.0\r\nUID:{name}\r\nFN:{name}\r\nEND:VCARD\r\n")
}

/// A home holding the vdir address book `book` with the cards `ada` and
/// `bob`, and the configuration of an account reading it.
fn vdir() -> (TempDir, String) {
    let home = TempDir::new().unwrap();
    let book = home.path().join("vdir").join("book");
    fs::create_dir_all(&book).unwrap();
    fs::write(book.join("ada.vcf"), card("Ada")).unwrap();
    fs::write(book.join("bob.vcf"), card("Bob")).unwrap();

    let config = home.path().join("config.toml");
    let toml = format!(
        "[accounts.a]\ndefault = true\nvdir.home-dir = {:?}\n",
        home.path().join("vdir").display().to_string(),
    );
    fs::write(&config, toml).unwrap();

    let config = config.display().to_string();
    (home, config)
}

/// Writes the card `eve` with no permission, so no read of it succeeds;
/// `false` when it still reads, as it does for root.
fn unreadable(home: &Path) -> bool {
    let path = home.join("vdir/book/eve.vcf");
    fs::write(&path, card("Eve")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
    fs::read(&path).is_err()
}

/// Runs the binary with `args` over the configuration `config`.
fn run(home: &Path, config: &str, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cardamum"))
        .args(["-c", config])
        .args(args)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home)
        .env_remove("XDG_CONFIG_DIRS")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

/// The JSON `output` printed once it exited 3.
fn partial(output: &Output) -> Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(3), "{stdout}");
    serde_json::from_str(&stdout).unwrap()
}

fn ids(json: &Value) -> Vec<&str> {
    let cards = json["cards"].as_array().unwrap();
    cards
        .iter()
        .map(|card| card["id"].as_str().unwrap())
        .collect()
}

#[test]
fn a_card_file_that_cannot_be_read_is_an_entry_the_others_listed() {
    let (home, config) = vdir();
    if !unreadable(home.path()) {
        return;
    }

    let output = run(
        home.path(),
        &config,
        &["--json", "card", "list", "-k", "book"],
    );
    let json = partial(&output);

    assert_eq!(ids(&json), ["ada", "bob"], "{json}");
    let errors = json["errors"].as_array().unwrap();
    assert_eq!(errors.len(), 1, "{json}");
    assert_eq!(errors[0]["id"], "eve");
    assert_eq!(errors[0]["code"], "card-unreadable");
    assert_eq!(errors[0]["error"], "Cannot read the card file");
    assert_eq!(errors[0]["sources"].as_array().unwrap().len(), 1, "{json}");
}

/// The whole directory is read for every page, so its entries print on
/// the first page alone.
#[test]
fn a_card_file_that_cannot_be_read_prints_on_the_first_page_alone() {
    let (home, config) = vdir();
    if !unreadable(home.path()) {
        return;
    }

    let args = ["--json", "card", "list", "-k", "book", "-s", "1", "-p", "2"];
    let output = run(home.path(), &config, &args);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(0), "{stdout}");
    let json: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(ids(&json), ["bob"], "{json}");
    assert_eq!(json["errors"], Value::Array(Vec::new()));
}

/// A card whose body does not parse is in `errors` alone, not also a row
/// of empty fields among the cards.
#[test]
fn a_card_that_does_not_parse_is_an_entry_alone() {
    let (home, config) = vdir();
    fs::write(home.path().join("vdir/book/bad.vcf"), "not a card").unwrap();

    let output = run(
        home.path(),
        &config,
        &["--json", "card", "list", "-k", "book"],
    );
    let json = partial(&output);

    assert_eq!(ids(&json), ["ada", "bob"], "{json}");
    let errors = json["errors"].as_array().unwrap();
    assert_eq!(errors.len(), 1, "{json}");
    assert_eq!(errors[0]["id"], "bad");
    assert_eq!(errors[0]["code"], "card-unreadable");
    assert_eq!(errors[0]["error"], "Cannot parse the card body as a vCard");
}

#[test]
fn a_card_that_is_not_utf8_is_an_entry_alone() {
    let (home, config) = vdir();
    let body = b"BEGIN:VCARD\r\nVERSION:4.0\r\nFN:\xff\xfe\r\nEND:VCARD\r\n";
    fs::write(home.path().join("vdir/book/bin.vcf"), body).unwrap();

    let output = run(
        home.path(),
        &config,
        &["--json", "card", "list", "-k", "book"],
    );
    let json = partial(&output);

    assert_eq!(ids(&json), ["ada", "bob"], "{json}");
    assert_eq!(json["errors"][0]["id"], "bin", "{json}");
    assert_eq!(json["errors"][0]["code"], "card-unreadable");
    assert_eq!(
        json["errors"][0]["error"],
        "Cannot read the card body as UTF-8"
    );
}

/// A card in `errors` keeps its place in the paging: the page holding it
/// names it, and the next page starts after it.
#[test]
fn a_card_that_does_not_parse_keeps_its_place_in_the_pages() {
    let (home, config) = vdir();
    fs::write(home.path().join("vdir/book/bad.vcf"), "not a card").unwrap();

    let args = ["--json", "card", "list", "-k", "book", "-s", "2"];
    let json = partial(&run(home.path(), &config, &args));
    assert_eq!(ids(&json), ["ada"], "{json}");
    assert_eq!(json["errors"][0]["id"], "bad", "{json}");
    assert_eq!(json["nextPage"], "2", "{json}");

    let args = ["--json", "card", "list", "-k", "book", "-s", "2", "-p", "2"];
    let output = run(home.path(), &config, &args);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(0), "{stdout}");
    let json: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(ids(&json), ["bob"], "{json}");
    assert_eq!(json["errors"], Value::Array(Vec::new()));
    assert_eq!(json["nextPage"], Value::Null, "{json}");
}

/// In text, the entry prints last, after the cards and one blank line,
/// and no row of empty cells stands for its card.
#[test]
fn the_text_listing_prints_its_errors_last() {
    let (home, config) = vdir();
    fs::write(home.path().join("vdir/book/bad.vcf"), "not a card").unwrap();

    let output = run(home.path(), &config, &["card", "list", "-k", "book"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(3), "{stdout}");

    let expected = "\
Account:      a (vdir)
Addressbook:  book
Page:         1, cards 1 to 2, the last

ada  -  -  Ada
bob  -  -  Bob

Error:  bad  card-unreadable  Cannot parse the card body as a vCard
";
    assert_eq!(stdout, expected);
}
