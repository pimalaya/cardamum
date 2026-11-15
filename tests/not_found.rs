//! # Not-found codes
//!
//! The built binary's failure object for a card or an address book the
//! account does not hold: exit 1, coded `card-not-found` or
//! `addressbook-not-found` under `--json`.

#![cfg(any(feature = "vdir", feature = "pimdir"))]

use std::{
    fs,
    path::Path,
    process::{Command, Output, Stdio},
};

use serde_json::Value;
use tempfile::TempDir;

/// Runs the binary with `args` over the configuration `toml`, in a home
/// of its own, its stdin closed.
fn run(home: &Path, toml: &str, args: &[&str]) -> Output {
    let config = home.join("config.toml");
    fs::write(&config, toml).unwrap();

    Command::new(env!("CARGO_BIN_EXE_cardamum"))
        .args(["--json", "-c", config.to_str().unwrap()])
        .args(args)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home)
        .env_remove("XDG_CONFIG_DIRS")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

/// The failure object `output` printed, once it exited 1.
fn failure(output: &Output) -> Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(1), "{stdout}");
    serde_json::from_str(&stdout).unwrap()
}

/// An account over a vdir home holding the empty address book `book`.
#[cfg(feature = "vdir")]
fn vdir(home: &Path) -> String {
    let vdir = home.join("vdir");
    fs::create_dir_all(vdir.join("book")).unwrap();
    format!(
        "[accounts.a]\ndefault = true\nvdir.root = {:?}\n",
        vdir.display().to_string(),
    )
}

#[cfg(feature = "vdir")]
#[test]
fn a_vdir_card_read_of_an_unknown_id_is_a_card_not_found() {
    let home = TempDir::new().unwrap();
    let toml = vdir(home.path());

    for verb in ["read", "delete"] {
        let output = run(home.path(), &toml, &["card", verb, "-k", "book", "nope"]);
        let json = failure(&output);
        assert_eq!(json["code"], "card-not-found", "{json}");

        // NOTE: cardamum's sentence, io-vdir's text below it, as calendula
        // words an item not found.
        assert_eq!(
            json["error"], "Card `nope` not found in address book `book`",
            "{json}"
        );
        let source = json["sources"][0].as_str().unwrap_or_default();
        assert!(source.contains("not found"), "{json}");
    }
}

#[cfg(feature = "vdir")]
#[test]
fn a_vdir_card_list_of_an_unknown_address_book_is_an_address_book_not_found() {
    let home = TempDir::new().unwrap();
    let toml = vdir(home.path());

    let output = run(home.path(), &toml, &["card", "list", "-k", "nope"]);
    let json = failure(&output);
    assert_eq!(json["code"], "addressbook-not-found", "{json}");
}

/// An account over a pimdir store holding `book`, an address book the
/// source `carddav` syncs, holding no card yet.
#[cfg(feature = "pimdir")]
fn pimdir(home: &Path) -> String {
    use io_pimdir::{
        change::PimdirWriteOp,
        client::PimdirStore,
        collection::{PimdirCheckpoint, PimdirCollectionId},
    };

    let root = home.join("store");
    fs::create_dir(&root).unwrap();

    let mut store = PimdirStore::open(&root).unwrap().for_source("carddav");
    store.ensure_collection("book", "text/vcard").unwrap();
    store
        .write(vec![PimdirWriteOp::SetCheckpoint {
            collection: PimdirCollectionId::from("book"),
            checkpoint: PimdirCheckpoint(Vec::new()),
        }])
        .unwrap();

    format!(
        "[accounts.a]\ndefault = true\npimdir.root = {:?}\n",
        root.display().to_string(),
    )
}

#[cfg(feature = "pimdir")]
#[test]
fn a_pimdir_card_read_of_an_unknown_id_is_a_card_not_found() {
    let home = TempDir::new().unwrap();
    let toml = pimdir(home.path());

    for id in ["42", "nope"] {
        let output = run(home.path(), &toml, &["card", "read", "-k", "book", id]);
        let json = failure(&output);
        assert_eq!(json["code"], "card-not-found", "{id}: {json}");
    }

    // NOTE: the sentence every backend words, as Himalaya words a message
    // not found in its mailbox.
    let output = run(home.path(), &toml, &["card", "read", "-k", "book", "42"]);
    let json = failure(&output);
    let error = "Card `42` not found in address book `book`";
    assert_eq!(json["error"], error, "{json}");
}

#[cfg(feature = "pimdir")]
#[test]
fn a_pimdir_card_list_of_an_unknown_address_book_is_an_address_book_not_found() {
    let home = TempDir::new().unwrap();
    let toml = pimdir(home.path());

    let output = run(home.path(), &toml, &["card", "list", "-k", "nope"]);
    let json = failure(&output);
    assert_eq!(json["code"], "addressbook-not-found", "{json}");
}
