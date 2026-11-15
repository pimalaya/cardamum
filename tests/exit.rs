//! # Exit contract
//!
//! The built binary's exit statuses beyond success and failure: a command
//! line the grammar refuses exits 2, its failure object coded
//! `usage-invalid` under `--json`, and a command that printed its result
//! without fully succeeding exits 3.

#![cfg(feature = "vdir")]

use std::{
    fs,
    path::Path,
    process::{Command, Output, Stdio},
};

use serde_json::Value;
use tempfile::TempDir;

/// Runs the binary with `args` in `home`, its stdin closed.
fn run(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cardamum"))
        .args(args)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home)
        .env_remove("XDG_CONFIG_DIRS")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn a_refused_command_line_under_json_prints_the_usage_failure_object() {
    let home = TempDir::new().unwrap();
    let output = run(home.path(), &["--json", "card", "list", "--nope"]);
    let stdout = text(&output.stdout);

    assert_eq!(output.status.code(), Some(2), "{stdout}");
    let json: Value = serde_json::from_str(&stdout).unwrap();
    let error = json["error"].as_str().unwrap();
    assert_eq!(json["code"], "usage-invalid", "{json}");
    assert!(error.starts_with("unexpected argument '--nope'"), "{error}");
    assert_eq!(json["sources"], Value::Array(Vec::new()), "{json}");
    assert!(output.stderr.is_empty(), "{}", text(&output.stderr));
}

#[test]
fn a_refused_command_line_without_json_is_clap_rendering_on_stderr() {
    let home = TempDir::new().unwrap();
    let output = run(home.path(), &["card", "list", "--nope"]);
    let stderr = text(&output.stderr);

    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(output.stdout.is_empty(), "{}", text(&output.stdout));
    assert!(stderr.starts_with("error: unexpected argument"), "{stderr}");
}

#[test]
fn a_failed_account_check_prints_its_report_and_exits_3() {
    let home = TempDir::new().unwrap();
    let config = home.path().join("config.toml");
    let missing = home.path().join("missing");
    let toml = format!(
        "[accounts.a]\ndefault = true\nvdir.home-dir = {:?}\n",
        missing.display().to_string(),
    );
    fs::write(&config, toml).unwrap();

    let args = ["--json", "-c", config.to_str().unwrap(), "account", "check"];
    let output = run(home.path(), &args);
    let stdout = text(&output.stdout);

    assert_eq!(output.status.code(), Some(3), "{stdout}");
    let json: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["backends"][0]["backend"], "vdir", "{json}");
    assert_eq!(json["backends"][0]["ok"], false, "{json}");
}

#[test]
fn a_listing_with_errors_prints_its_result_and_exits_3() {
    let home = TempDir::new().unwrap();
    let book = home.path().join("vdir").join("book");
    fs::create_dir_all(&book).unwrap();
    fs::write(book.join("bad.vcf"), "not a card").unwrap();

    let config = home.path().join("config.toml");
    let toml = format!(
        "[accounts.a]\ndefault = true\nvdir.home-dir = {:?}\n",
        home.path().join("vdir").display().to_string(),
    );
    fs::write(&config, toml).unwrap();

    let config = config.to_str().unwrap();
    let args = ["--json", "-c", config, "card", "list", "-k", "book"];
    let output = run(home.path(), &args);
    let stdout = text(&output.stdout);

    assert_eq!(output.status.code(), Some(3), "{stdout}");
    let json: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["cards"], Value::Array(Vec::new()), "{json}");
    assert_eq!(json["errors"][0]["id"], "bad", "{json}");
    assert_eq!(json["errors"][0]["code"], "card-unreadable", "{json}");
}
