//! # pimdir refusals
//!
//! The refusals of the pimdir backend carry the codes Himalaya and the
//! pimdir binary give them, run as the built binary over a store io-pimdir
//! fills: an unknown queue row, a cancellation no terminal can confirm, a
//! page token the command did not print, and an account the store holds
//! several of, or none of.

#![cfg(feature = "pimdir")]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::Mutex,
};

use io_pimdir::{
    change::PimdirWriteOp,
    client::PimdirStore,
    collection::{PimdirCheckpoint, PimdirCollectionId},
};
use serde_json::Value;
use tempfile::TempDir;

const CARD: &str = "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:jane\r\nFN:Jane Doe\r\nEND:VCARD\r\n";

/// Held while a test opens a store as its owner or spawns the binary, so
/// no child spawned by a test running beside inherits a lock the owner
/// takes, which would read as a store another process owns.
static STORE: Mutex<()> = Mutex::new(());

/// Declares the address book `book` the source `carddav` syncs, grouped
/// under `account` when one is named.
fn synced(root: &Path, account: Option<&str>, book: &str) {
    let _held = STORE.lock().unwrap_or_else(|err| err.into_inner());
    let mut store = PimdirStore::open(root).unwrap().for_source("carddav");
    if let Some(account) = account {
        store = store.for_account(account);
    }
    store.ensure_collection(book, "text/vcard").unwrap();
    store
        .write(vec![PimdirWriteOp::SetCheckpoint {
            collection: PimdirCollectionId::from(book),
            checkpoint: PimdirCheckpoint(Vec::new()),
        }])
        .unwrap();
}

/// The configuration reaching the store at `root`, `account` as its
/// `pimdir.account`.
fn config(dir: &Path, root: &Path, account: Option<&str>) -> PathBuf {
    let config = dir.join("config.toml");
    let mut lines = format!("[accounts.a]\ndefault = true\npimdir.root = {root:?}\n");
    if let Some(account) = account {
        lines.push_str(&format!("pimdir.account = {account:?}\n"));
    }
    fs::write(&config, lines).unwrap();
    config
}

/// A store holding the address book `book` of no account, and the
/// configuration reaching it.
fn store(dir: &Path) -> PathBuf {
    let root = dir.join("store");
    fs::create_dir(&root).unwrap();
    synced(&root, None, "book");
    config(dir, &root, None)
}

/// Runs the binary with `args` under `--json`, its stdin closed.
fn run(config: &Path, args: &[&str]) -> Output {
    let _held = STORE.lock().unwrap_or_else(|err| err.into_inner());
    let home = config.parent().unwrap();
    Command::new(env!("CARGO_BIN_EXE_cardamum"))
        .arg("-c")
        .arg(config)
        .arg("--json")
        .args(args)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home)
        .env_remove("XDG_CONFIG_DIRS")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

/// What a run printed, checked to exit with `code`.
fn printed(config: &Path, args: &[&str], code: i32) -> Value {
    let output = run(config, args);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(code), "{args:?}: {stdout}");
    serde_json::from_str(stdout.trim()).unwrap()
}

#[test]
fn the_refusals_of_a_store_carry_their_codes() {
    let dir = TempDir::new().unwrap();
    let config = store(dir.path());

    let cases: [(&[&str], &str); 3] = [
        (
            &["pimdir", "queue", "cancel", "999", "--yes"],
            "row-not-found",
        ),
        (&["pimdir", "queue", "cancel", "999"], "row-not-found"),
        (
            &["pimdir", "search", "--page-token", "zzz", "jane"],
            "page-token-invalid",
        ),
    ];

    for (args, code) in cases {
        let err = printed(&config, args, 1);
        assert_eq!(err["code"], code, "{args:?}: {err}");
    }
}

/// A cancellation asked with no terminal to answer and no `--yes` is
/// refused before anything is opened, the row left queued.
#[test]
fn a_cancellation_no_terminal_confirms_is_refused() {
    let dir = TempDir::new().unwrap();
    let config = store(dir.path());
    printed(&config, &["card", "add", "-k", "book", CARD], 0);
    let queue = printed(&config, &["pimdir", "queue", "list"], 0);
    let row = queue["cards"][0]["row"].to_string();

    let err = printed(&config, &["pimdir", "queue", "cancel", &row], 1);
    assert_eq!(err["code"], "confirmation-required", "{err}");

    let queue = printed(&config, &["pimdir", "queue", "list"], 0);
    assert_eq!(queue["cards"].as_array().unwrap().len(), 1, "{queue}");
}

/// An account the store does not hold is refused rather than read as
/// empty, and several accounts with none named are refused rather than
/// one guessed.
#[test]
fn the_account_is_the_one_named_or_refused() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("store");
    fs::create_dir(&root).unwrap();
    synced(&root, Some("work"), "work/book");
    synced(&root, Some("home"), "home/book");

    let named = config(dir.path(), &root, Some("nope"));
    let err = printed(&named, &["addressbook", "list"], 1);
    assert_eq!(err["code"], "store-account-unknown", "{err}");

    let unset = config(dir.path(), &root, None);
    let err = printed(&unset, &["addressbook", "list"], 1);
    assert_eq!(err["code"], "store-account-ambiguous", "{err}");
}
