//! # Capabilities
//!
//! The built binary over a vdir account, which serves no `--if-match`: a
//! verb its backend does not serve is refused before anything is read or
//! written, `account capability` lists what it serves, and the help of a
//! unified leaf says which backend serves what.

#![cfg(feature = "vdir")]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use serde_json::Value;
use tempfile::TempDir;

const CARD: &str = "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:ada\r\nFN:Ada\r\nEND:VCARD\r\n";

/// A home holding a vdir addressbook `personal` with one card, and the
/// configuration of an account reading it.
fn vdir() -> (TempDir, PathBuf) {
    let home = TempDir::new().unwrap();
    let book = home.path().join("contacts").join("personal");
    fs::create_dir_all(&book).unwrap();
    fs::write(book.join("ada.vcf"), CARD).unwrap();

    let config = home.path().join("config.toml");
    let contacts = home.path().join("contacts");
    let toml = format!(
        "[accounts.example]\ndefault = true\nvdir.home-dir = {:?}\n",
        contacts.display().to_string(),
    );
    fs::write(&config, toml).unwrap();

    (home, config)
}

fn run(home: &Path, config: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cardamum"))
        .arg("-c")
        .arg(config)
        .args(args)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home)
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

#[test]
fn an_unserved_verb_is_refused_before_anything_is_touched() {
    let (home, config) = vdir();
    let args = [
        "--json",
        "card",
        "delete",
        "-k",
        "personal",
        "ada",
        "--if-match",
        "x",
    ];
    let output = run(home.path(), &config, &args);

    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["code"], "capability-missing", "{json}");
    let error = json["error"].as_str().unwrap();
    assert!(error.contains("`card.remove.if-match`"), "{error}");

    let card = home.path().join("contacts/personal/ada.vcf");
    assert_eq!(fs::read_to_string(card).unwrap(), CARD);
}

#[test]
fn the_account_lists_what_its_backend_serves() {
    let (home, config) = vdir();
    let output = run(home.path(), &config, &["--json", "account", "capability"]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["account"], "example");
    assert_eq!(json["backend"], "vdir");

    let capabilities = json["capabilities"].as_array().unwrap();
    let level = |name: &str| {
        let capability = capabilities.iter().find(|c| c["name"] == name).unwrap();
        capability["level"].as_str().unwrap().to_string()
    };
    assert_eq!(level("card.add"), "full");
    assert_eq!(level("card.remove.if-match"), "none");
}

#[test]
fn a_unified_leaf_help_names_the_backends_serving_it() {
    let (home, config) = vdir();
    let output = run(home.path(), &config, &["card", "delete", "--help"]);
    let help = String::from_utf8_lossy(&output.stdout);

    assert!(help.contains("Backends:"), "{help}");
    assert!(
        help.contains("card.remove.if-match: partial on carddav;"),
        "{help}"
    );
}
