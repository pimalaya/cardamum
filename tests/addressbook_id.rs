//! # Address book id
//!
//! The built binary creating an address book under the id `--id` chooses:
//! a vdir account names its directory after it, the name staying the
//! display name, and a backend minting its own ids refuses the flag
//! before connecting, coded `capability-missing`.

#![cfg(any(feature = "vdir", feature = "jmap"))]

use std::{
    fs,
    path::Path,
    process::{Command, Output, Stdio},
};

use serde_json::Value;
use tempfile::TempDir;

/// Runs the binary under `--json` with `args` over the configuration
/// `toml`, written in `dir`.
fn run(dir: &Path, toml: &str, args: &[&str]) -> Output {
    let config = dir.join("config.toml");
    fs::write(&config, toml).unwrap();

    Command::new(env!("CARGO_BIN_EXE_cardamum"))
        .arg("--json")
        .arg("-c")
        .arg(config)
        .args(args)
        .env("HOME", dir)
        .env("XDG_CONFIG_HOME", dir)
        .env_remove("XDG_CONFIG_DIRS")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

/// `--id` names the vdir directory, the name its display name; the id
/// taken a second time is a duplicate naming it.
#[cfg(feature = "vdir")]
#[test]
fn a_vdir_address_book_takes_the_id_given() {
    let dir = TempDir::new().unwrap();
    let home = dir.path().join("home");
    fs::create_dir_all(&home).unwrap();
    let toml = format!(
        "[accounts.perso]\ndefault = true\nvdir.root = {:?}\n",
        home.display().to_string()
    );
    let args = ["addressbook", "create", "My Work", "--id", "work"];

    let output = run(dir.path(), &toml, &args);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(0), "{stdout}");
    let json: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["id"], "work", "{json}");
    assert_eq!(json["name"], "My Work", "{json}");

    let name = fs::read_to_string(home.join("work").join("displayname")).unwrap();
    assert_eq!(name.trim_end(), "My Work");

    let output = run(dir.path(), &toml, &args);
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["code"], "addressbook-duplicate", "{json}");
    let duplicate = "Cannot create address book `work`: it exists already";
    assert_eq!(json["error"], duplicate, "{json}");
}

/// A JMAP server mints the id of an address book, so `--id` is refused
/// before anything connects: the server named does not exist.
#[cfg(feature = "jmap")]
#[test]
fn a_backend_minting_its_ids_refuses_the_id() {
    let dir = TempDir::new().unwrap();
    let toml = "[accounts.perso]\ndefault = true\n\
                jmap.server = \"https://jmap.invalid/jmap/session\"\n\
                jmap.auth.basic.username = \"alice\"\n\
                jmap.auth.basic.password.command = \"exit 127\"\n";
    let args = ["addressbook", "create", "Work", "--id", "work"];

    let output = run(dir.path(), toml, &args);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(1), "{stdout}");
    let json: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["code"], "capability-missing", "{json}");
    let error = json["error"].as_str().unwrap();
    assert!(error.contains("`addressbook.create.id`"), "{error}");
}
