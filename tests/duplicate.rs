//! # Duplicate address book
//!
//! The built binary creating an address book its vdir home holds already:
//! `addressbook create` and `vdir create` fail coded
//! `addressbook-duplicate`, exit 1, and leave it as it was; `vdir item
//! move` and `copy` onto an item of the same id fail coded `item-duplicate`
//! and leave both items as they were.

#![cfg(feature = "vdir")]

use std::{
    fs,
    path::Path,
    process::{Command, Output, Stdio},
};

use serde_json::Value;
use tempfile::TempDir;

/// A directory holding the vdir home `home`, its address book `work`
/// named `Work` and described, and the configuration file.
fn account() -> TempDir {
    let dir = TempDir::new().unwrap();
    let work = dir.path().join("home").join("work");
    fs::create_dir_all(&work).unwrap();
    fs::write(work.join("displayname"), "Work").unwrap();
    fs::write(work.join("description"), "Colleagues").unwrap();

    let config = format!(
        "[accounts.perso]\ndefault = true\nvdir.root = {:?}\n",
        dir.path().join("home").display().to_string()
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

#[test]
fn an_address_book_created_twice_is_a_duplicate() {
    let dir = account();
    let work = dir.path().join("home").join("work");
    let creates: [&[&str]; 3] = [
        &["addressbook", "create", "work"],
        &["addressbook", "create", "work", "-d", "Other"],
        &["vdir", "create", "work", "-n", "Other"],
    ];

    for args in creates {
        let output = run(dir.path(), args);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(output.status.code(), Some(1), "{args:?}: {stdout}");

        let json: Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(json["code"], "addressbook-duplicate", "{json}");
        assert_eq!(
            json["error"], "Cannot create address book `work`: it exists already",
            "{json}"
        );

        let name = fs::read_to_string(work.join("displayname")).unwrap();
        let description = fs::read_to_string(work.join("description")).unwrap();
        assert_eq!(
            (name.as_str(), description.as_str()),
            ("Work", "Colleagues")
        );
    }
}

#[test]
fn an_item_moved_or_copied_onto_a_same_id_item_is_a_duplicate() {
    let dir = account();
    let home = dir.path().join("home");
    let (work, perso) = (home.join("work"), home.join("perso"));
    fs::create_dir_all(&perso).unwrap();
    fs::write(
        work.join("x.vcf"),
        "BEGIN:VCARD\r\nFN:Work\r\nEND:VCARD\r\n",
    )
    .unwrap();
    fs::write(
        perso.join("x.vcf"),
        "BEGIN:VCARD\r\nFN:Perso\r\nEND:VCARD\r\n",
    )
    .unwrap();

    for verb in ["move", "copy"] {
        let output = run(dir.path(), &["vdir", "item", verb, "work", "perso", "x"]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(output.status.code(), Some(1), "{verb}: {stdout}");

        let json: Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(json["code"], "item-duplicate", "{json}");
        assert_eq!(
            json["error"],
            format!("Cannot {verb} item `x` into `perso`: it holds an item of that id already"),
            "{json}"
        );

        let kept = fs::read_to_string(perso.join("x.vcf")).unwrap();
        assert!(kept.contains("FN:Perso"), "{verb}: {kept}");
        assert!(work.join("x.vcf").exists(), "{verb}");
    }
}
