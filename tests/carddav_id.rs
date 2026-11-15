//! # CardDAV ids
//!
//! The built binary naming, on a CardDAV account, an address book or a card
//! by an id no CardDAV resource can have, one addressing the home set, its
//! parent or the server root: every command fails before the account's
//! secret is resolved or anything connects, coded as an address book or a
//! card the server does not hold.

#![cfg(feature = "carddav")]

use std::{
    fs,
    path::Path,
    process::{Command, Output, Stdio},
};

use serde_json::Value;
use tempfile::TempDir;

/// A directory holding the configuration of a CardDAV account whose home
/// set no server answers, its password command creating `spawned`: the
/// file says the account was opened.
fn account() -> TempDir {
    let dir = TempDir::new().unwrap();
    let spawned = dir.path().join("spawned").display().to_string();

    let config = format!(
        "[accounts.dav]\n\
         default = true\n\
         carddav.home = \"http://127.0.0.1:9/dav/\"\n\
         carddav.auth.basic.username = \"alice\"\n\
         carddav.auth.basic.password.command = [\"touch\", {spawned:?}]\n"
    );
    fs::write(dir.path().join("config.toml"), config).unwrap();

    dir
}

/// Runs the binary under `--json` with `args` over the account in `dir`.
fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cardamum"))
        .arg("--json")
        .arg("-c")
        .arg(dir.join("config.toml"))
        .args(args)
        .env("HOME", dir)
        .env("XDG_CONFIG_HOME", dir)
        .env_remove("XDG_CONFIG_DIRS")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

/// Fails unless `args` exit 1 with the failure object coded `code` saying
/// `error`, the account never opened.
fn refused(dir: &Path, args: &[&str], code: Value, error: &str) {
    let out = run(dir, args);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap_or_else(|_| panic!("{stdout}"));

    assert_eq!(out.status.code(), Some(1), "{args:?}: {json}");
    assert_eq!(json["code"], code, "{args:?}: {json}");
    assert_eq!(json["error"], error, "{args:?}: {json}");
    assert!(!dir.join("spawned").exists(), "{args:?} opened the account");
}

#[test]
fn an_addressbook_id_out_of_the_home_is_not_found() {
    let dir = account();
    let code = Value::from("addressbook-not-found");

    for id in ["..", ".", "/", "a/..", "%2e%2e", "a//b"] {
        let error = format!("Addressbook `{id}` not found");

        let commands: &[&[&str]] = &[
            &["addressbook", "delete", id],
            &["addressbook", "update", id, "-n", "Renamed"],
            &["card", "list", "-k", id],
            &["card", "read", "-k", id, "c1.vcf"],
            &["card", "add", "-k", id, "--full-name", "Bob"],
            &["card", "update", "-k", id, "c1.vcf", "--full-name", "Bob"],
            &["card", "delete", "-k", id, "c1.vcf"],
            &["carddav", "propfind", id],
            &["carddav", "proppatch", id, "-n", "Renamed"],
            &["carddav", "get", id, "c1.vcf"],
            &[
                "carddav",
                "put",
                id,
                "c1.vcf",
                "BEGIN:VCARD\r\nEND:VCARD\r\n",
            ],
            &["carddav", "delete", id],
            &["carddav", "delete", id, "c1.vcf"],
            &["carddav", "report", "query", id],
            &["carddav", "report", "multiget", id, "c1.vcf"],
            &["carddav", "report", "sync", id],
            &["carddav", "report", "raw", id, "<x/>"],
        ];

        for args in commands {
            refused(dir.path(), args, code.clone(), &error);
        }
    }
}

#[test]
fn a_card_id_out_of_its_addressbook_is_not_found() {
    let dir = account();
    let code = Value::from("card-not-found");

    for id in ["..", ".", "", "/", "a/../..", "%2E%2E", "a/"] {
        let error = format!("Card `{id}` not found in address book `book`");

        let commands: &[&[&str]] = &[
            &["card", "read", "-k", "book", id],
            &["card", "update", "-k", "book", id, "--full-name", "Bob"],
            &["card", "delete", "-k", "book", id],
            &["carddav", "get", "book", id],
            &["carddav", "put", "book", id, "BEGIN:VCARD\r\nEND:VCARD\r\n"],
            &["carddav", "delete", "book", id],
            &["carddav", "report", "multiget", "book", "c1.vcf", id],
        ];

        for args in commands {
            refused(dir.path(), args, code.clone(), &error);
        }
    }
}

#[test]
fn a_new_addressbook_name_out_of_the_home_is_refused() {
    let dir = account();

    for name in ["..", "/", "a/.."] {
        let error = format!("Addressbook name `{name}` names no CardDAV collection");
        refused(
            dir.path(),
            &["addressbook", "create", name],
            Value::Null,
            &error,
        );
        refused(dir.path(), &["carddav", "mkcol", name], Value::Null, &error);
    }

    let error = "Addressbook name cannot be empty";
    refused(
        dir.path(),
        &["addressbook", "create", ""],
        Value::Null,
        error,
    );
}

#[test]
fn a_clean_id_goes_on_to_open_the_account() {
    let dir = account();

    let out = run(dir.path(), &["card", "read", "-k", "My Book", "c1.vcf"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(dir.path().join("spawned").exists());
}
