//! # Addressbook not found
//!
//! The built binary naming an address book its vdir home does not hold:
//! every command fails coded `addressbook-not-found` and exits 1, reading,
//! creating and removing nothing, an id reaching out of the home included.

#![cfg(feature = "vdir")]

use std::{
    fs,
    path::Path,
    process::{Command, Output, Stdio},
};

use serde_json::Value;
use tempfile::TempDir;

/// A directory holding `outer`, which holds the vdir home `home`, its
/// address book `friends` with the one card `a`, and the directory `keep`
/// beside the home with one file; and the configuration file.
fn account() -> TempDir {
    let dir = TempDir::new().unwrap();
    let outer = dir.path().join("outer");
    let friends = outer.join("home").join("friends");
    fs::create_dir_all(&friends).unwrap();
    fs::create_dir(outer.join("keep")).unwrap();
    fs::write(outer.join("keep").join("file"), "kept").unwrap();

    let vcf = "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:a@example.org\r\nFN:Ann\r\nEND:VCARD\r\n";
    fs::write(friends.join("a.vcf"), vcf).unwrap();

    let config = format!(
        "[accounts.perso]\ndefault = true\nvdir.root = {:?}\n",
        outer.join("home").display().to_string()
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

/// The failure object `out` printed, once it exited 1.
fn failure(out: &Output) -> Value {
    let stdout = String::from_utf8_lossy(&out.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap_or_else(|_| panic!("{stdout}"));

    assert_eq!(out.status.code(), Some(1), "{json}");
    json
}

/// Fails unless `out` is the failure object coded `addressbook-not-found`
/// saying `error`, exiting 1.
fn refused(out: &Output, error: &str) {
    let json = failure(out);

    assert_eq!(json["code"], "addressbook-not-found", "{json}");
    assert_eq!(json["error"], error, "{json}");
}

/// The paths under `dir`, sorted, relative to it.
fn tree(dir: &Path) -> Vec<String> {
    let mut paths = Vec::new();
    let mut pending = vec![dir.to_path_buf()];

    while let Some(path) = pending.pop() {
        for entry in fs::read_dir(&path).unwrap() {
            let path = entry.unwrap().path();
            let relative = path.strip_prefix(dir).unwrap();
            paths.push(relative.display().to_string());

            if path.is_dir() {
                pending.push(path);
            }
        }
    }

    paths.sort();
    paths
}

/// Fails unless the file beside the home is still there.
fn kept(dir: &Path) {
    let file = dir.join("outer").join("keep").join("file");
    assert_eq!(fs::read_to_string(&file).unwrap(), "kept");
}

#[test]
fn every_unified_command_refuses_an_unknown_addressbook() {
    let dir = account();
    let before = tree(dir.path());

    let commands: &[&[&str]] = &[
        &["card", "list", "-k", "nope"],
        &["card", "read", "-k", "nope", "a"],
        &["card", "create", "-k", "nope", "--full-name", "Bob"],
        &["card", "update", "-k", "nope", "a", "--full-name", "Bob"],
        &["card", "delete", "-k", "nope", "a"],
        &["addressbook", "update", "nope", "-n", "Renamed"],
        &["addressbook", "update", "-k", "nope", "-n", "Renamed"],
        &["addressbook", "delete", "nope"],
        &["addressbook", "delete", "-k", "nope"],
    ];

    for args in commands {
        refused(&run(dir.path(), args), "Addressbook `nope` not found");
    }

    assert_eq!(tree(dir.path()), before);
}

#[test]
fn an_addressbook_reaching_out_of_the_home_is_not_found() {
    let dir = account();
    let before = tree(dir.path());

    for id in ["..", "../keep", "friends/..", ".", "friends/", ""] {
        let error = format!("Addressbook `{id}` not found");

        let out = run(dir.path(), &["addressbook", "delete", id]);
        refused(&out, &error);
        kept(dir.path());

        let error = match id {
            "" => "Addressbook id cannot be empty",
            _ => &error,
        };

        refused(&run(dir.path(), &["card", "list", "-k", id]), error);
        refused(&run(dir.path(), &["card", "read", "-k", id, "a"]), error);
        refused(&run(dir.path(), &["card", "delete", "-k", id, "a"]), error);
    }

    assert_eq!(tree(dir.path()), before);
}

#[test]
fn the_vdir_commands_refuse_an_unknown_collection() {
    let dir = account();
    let before = tree(dir.path());

    let commands: &[(&str, &[&str])] = &[
        ("nope", &["vdir", "delete", "nope"]),
        ("", &["vdir", "delete", ""]),
        (".", &["vdir", "delete", "."]),
        ("..", &["vdir", "delete", ".."]),
        ("../keep", &["vdir", "delete", "../keep"]),
        ("friends/..", &["vdir", "delete", "friends/.."]),
        ("nope", &["vdir", "rename", "nope", "renamed"]),
        ("..", &["vdir", "rename", "..", "renamed"]),
        ("..", &["vdir", "item", "list", ".."]),
        ("..", &["vdir", "item", "get", "..", "a"]),
        ("..", &["vdir", "item", "delete", "..", "a"]),
    ];

    for (id, args) in commands {
        let out = run(dir.path(), args);
        refused(&out, &format!("Collection `{id}` not found"));
        kept(dir.path());
    }

    assert_eq!(tree(dir.path()), before);
}

#[test]
fn a_new_name_reaching_out_of_the_home_is_refused() {
    let dir = account();
    let before = tree(dir.path());

    let commands: &[&[&str]] = &[
        &["addressbook", "create", ".."],
        &["addressbook", "create", "../keep"],
        &["addressbook", "create", "friends/new"],
        &["vdir", "create", ".."],
        &["vdir", "create", "../escape"],
        &["vdir", "rename", "friends", "../escape"],
        &["vdir", "rename", "friends", ".."],
    ];

    for args in commands {
        let json = failure(&run(dir.path(), args));
        assert_eq!(json["code"], Value::Null, "{args:?}: {json}");
    }

    assert_eq!(tree(dir.path()), before);
}

#[test]
fn an_addressbook_the_home_holds_is_read_and_deleted_alone() {
    let dir = account();

    let out = run(dir.path(), &["card", "list", "-k", "friends"]);
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(out.status.code(), Some(0), "{json}");
    assert_eq!(json["cards"][0]["id"], "a", "{json}");

    let out = run(dir.path(), &["addressbook", "delete", "friends"]);
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(out.status.code(), Some(0), "{json}");
    assert_eq!(json["id"], "friends", "{json}");

    assert_eq!(
        tree(dir.path()),
        [
            "config.toml",
            "outer",
            "outer/home",
            "outer/keep",
            "outer/keep/file",
        ]
    );
}
