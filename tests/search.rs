//! # Card search
//!
//! The built binary over a vdir account: `card search` matches every
//! card of the address books in scope with the pimdir query language,
//! refusing a malformed query and a term only a pimdir store answers
//! before anything is read.

#![cfg(feature = "vdir")]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use serde_json::Value;
use tempfile::TempDir;

/// A vCard 4.0 of `uid` and `name`, then `lines`.
fn card(uid: &str, name: &str, lines: &str) -> String {
    format!("BEGIN:VCARD\r\nVERSION:4.0\r\nUID:{uid}\r\nFN:{name}\r\n{lines}END:VCARD\r\n")
}

/// A home holding the vdir address books `personal` and `work`, and the
/// configuration of an account reading them.
fn vdir() -> (TempDir, PathBuf) {
    let home = TempDir::new().unwrap();
    let contacts = home.path().join("contacts");
    let cards = [
        (
            "personal",
            "elodie",
            card("elodie", "Élodie Martin", "EMAIL:elodie@example.org\r\n"),
        ),
        (
            "personal",
            "bob",
            card("bob", "Bob Durand", "TEL:+33 1 00 00 00 00\r\n"),
        ),
        (
            "work",
            "ada",
            card(
                "ada",
                "Ada Lovelace",
                "EMAIL:ada@example.org\r\nORG:Engines\r\n",
            ),
        ),
    ];
    for (book, id, body) in cards {
        let dir = contacts.join(book);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(format!("{id}.vcf")), body).unwrap();
    }

    let config = home.path().join("config.toml");
    let toml = format!(
        "[accounts.example]\ndefault = true\nvdir.root = {:?}\n",
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

fn json(home: &Path, config: &Path, args: &[&str]) -> (Option<i32>, Value) {
    let mut all = vec!["--json", "card", "search"];
    all.extend(args);
    let output = run(home, config, &all);
    let json = serde_json::from_slice(&output.stdout).unwrap();
    (output.status.code(), json)
}

fn names(json: &Value) -> Vec<&str> {
    json["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|card| card["fullName"].as_str().unwrap())
        .collect()
}

/// A query without its diacritics finds the accented name, as SEARCH §5
/// folds it, over every address book, in name order.
#[test]
fn a_query_matches_every_address_book_folded() {
    let (home, config) = vdir();

    let (code, found) = json(home.path(), &config, &["elodie"]);
    assert_eq!(code, Some(0), "{found}");
    assert_eq!(names(&found), ["Élodie Martin"]);
    assert_eq!(found["cards"][0]["addressbookId"], "personal");
    assert_eq!(found["complete"], true);
    assert!(found["nextPage"].is_null(), "{found}");

    let (_, found) = json(home.path(), &config, &["email:example.org"]);
    assert_eq!(names(&found), ["Ada Lovelace", "Élodie Martin"]);

    let (_, found) = json(home.path(), &config, &["-k", "work", "kind:contact"]);
    assert_eq!(names(&found), ["Ada Lovelace"]);

    let (_, found) = json(home.path(), &config, &["has:phone"]);
    assert_eq!(names(&found), ["Bob Durand"]);
}

/// A page holds `--page-size` hits, its `nextPage` the next number.
#[test]
fn the_hits_page_by_number() {
    let (home, config) = vdir();

    let (_, first) = json(home.path(), &config, &["-s", "2", "kind:contact"]);
    assert_eq!(names(&first), ["Ada Lovelace", "Bob Durand"]);
    assert_eq!(first["nextPage"], "2");

    let (_, second) = json(
        home.path(),
        &config,
        &["-s", "2", "--page-token", "2", "kind:contact"],
    );
    assert_eq!(names(&second), ["Élodie Martin"]);
    assert!(second["nextPage"].is_null(), "{second}");

    let (code, bad) = json(home.path(), &config, &["--page-token", "x", "kind:contact"]);
    assert_eq!(code, Some(1));
    assert_eq!(bad["code"], "page-token-invalid", "{bad}");
}

/// A malformed query is `query-refused`, a store term
/// `query-unsupported` naming it, relevance `capability-missing`, and an
/// unknown address book `addressbook-not-found` naming the ones held.
#[test]
fn a_query_vdir_cannot_answer_is_refused() {
    let (home, config) = vdir();

    let (code, refused) = json(home.path(), &config, &["frm:alice"]);
    assert_eq!(code, Some(1));
    assert_eq!(refused["code"], "query-refused", "{refused}");

    let (code, unsupported) = json(home.path(), &config, &["tag:work", "alice"]);
    assert_eq!(code, Some(1));
    assert_eq!(unsupported["code"], "query-unsupported", "{unsupported}");
    let error = unsupported["error"].as_str().unwrap();
    assert!(error.contains("`tag:work` at 0..8"), "{error}");

    let (code, ranked) = json(home.path(), &config, &["--sort", "relevance", "alice"]);
    assert_eq!(code, Some(1));
    assert_eq!(ranked["code"], "capability-missing", "{ranked}");

    let (code, missing) = json(home.path(), &config, &["-k", "nope", "alice"]);
    assert_eq!(code, Some(1));
    assert_eq!(missing["code"], "addressbook-not-found", "{missing}");
    let error = missing["error"].as_str().unwrap();
    assert!(error.contains("personal, work"), "{error}");
}

/// The text lists each hit on one line, its address book after its id.
#[test]
fn a_hit_prints_on_one_line() {
    let (home, config) = vdir();
    let output = run(home.path(), &config, &["card", "search", "durand"]);
    let text = String::from_utf8(output.stdout).unwrap();

    assert!(text.contains("Query:"), "{text}");
    assert!(
        text.contains("bob  personal  -  +33 1 00 00 00 00  Bob Durand"),
        "{text}"
    );
}
