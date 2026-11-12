//! # REPL
//!
//! The built binary's `repl` over a vdir account: lines run as commands
//! over one account until `quit`, `--json` switches one line, and what
//! cannot run inside a session (a session, the composer, stdin) is
//! refused by name while the session goes on.

#![cfg(feature = "vdir")]

use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
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

/// Runs a session reading `lines`, returning its stdout lines.
fn session(lines: &str) -> Vec<String> {
    let (home, config) = vdir();
    let mut child = Command::new(env!("CARGO_BIN_EXE_cardamum"))
        .arg("-c")
        .arg(&config)
        .arg("repl")
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .take()
        .unwrap()
        .write_all(lines.as_bytes())
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    stdout.lines().map(String::from).collect()
}

#[test]
fn a_line_runs_over_the_account_and_json_switches_one_line() {
    let lines = session("card list -k personal\n--json card read -k personal ada\nquit\n");

    assert!(
        lines.iter().any(|line| line.starts_with("ada  ")),
        "{lines:?}"
    );
    let json: Value = serde_json::from_str(lines.last().unwrap()).unwrap();
    assert_eq!(json["id"], "ada");
}

#[test]
fn what_cannot_run_in_a_session_is_refused_and_the_session_goes_on() {
    let lines = session(
        "--json repl\n\
         --json card create -i -k personal\n\
         --json card create -k personal -\n\
         --json addressbook list\n",
    );
    let json: Vec<Value> = lines
        .iter()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();

    assert!(
        json[0]["error"]
            .as_str()
            .unwrap()
            .contains("inside a session")
    );
    assert!(json[1]["error"].as_str().unwrap().contains("composer (-i)"));
    assert_eq!(json[2]["code"], "stdin-taken");
    assert_eq!(json[3]["addressbooks"][0]["id"], "personal");
}
