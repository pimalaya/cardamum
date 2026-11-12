//! # MCP
//!
//! The built binary's `mcp` server over a vdir account: it introduces
//! itself, lists the reading tools alone unless `--allow-write`, answers a
//! call with the command's JSON, and takes a card as text, never as a
//! local path.

#![cfg(feature = "vdir")]

use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

use serde_json::{Value, json};
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

/// Runs a server with `flags` in `home`, sends `requests` after the
/// handshake, and returns its answers keyed by request id.
fn serve(home: &TempDir, config: &PathBuf, flags: &[&str], requests: &[Value]) -> Vec<Value> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cardamum"))
        .arg("-c")
        .arg(config)
        .arg("mcp")
        .args(flags)
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let initialize = json!({
        "jsonrpc": "2.0", "id": 0, "method": "initialize",
        "params": {"protocolVersion": "2025-06-18", "capabilities": {},
                   "clientInfo": {"name": "test", "version": "0"}},
    });
    let initialized = json!({"jsonrpc": "2.0", "method": "notifications/initialized"});

    let mut stdin = child.stdin.take().unwrap();
    for message in [&initialize, &initialized].into_iter().chain(requests) {
        writeln!(stdin, "{message}").unwrap();
    }
    drop(stdin);

    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    stdout
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn call(id: u64, name: &str, arguments: Value) -> Value {
    json!({
        "jsonrpc": "2.0", "id": id, "method": "tools/call",
        "params": {"name": name, "arguments": arguments},
    })
}

fn tool_names(answer: &Value) -> Vec<&str> {
    let tools = answer["result"]["tools"].as_array().unwrap();
    tools
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect()
}

#[test]
fn the_server_introduces_itself_and_lists_reads_alone() {
    let (home, config) = vdir();
    let list = json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list"});
    let answers = serve(&home, &config, &[], &[list]);

    assert_eq!(answers[0]["result"]["serverInfo"]["name"], "cardamum");
    assert!(answers[0]["result"]["instructions"].is_string());
    assert_eq!(
        tool_names(&answers[1]),
        [
            "account_capability",
            "addressbook_list",
            "card_list",
            "card_read"
        ]
    );
}

#[test]
fn a_call_answers_the_command_json() {
    let (home, config) = vdir();
    let list = call(1, "card_list", json!({"addressbook": "personal"}));
    let answers = serve(&home, &config, &[], &[list]);

    let result = &answers[1]["result"];
    assert_ne!(result["isError"], true, "{result}");
    assert_eq!(result["structuredContent"]["cards"][0]["id"], "ada");
    assert!(result["structuredContent"]["nextPage"].is_null());
}

#[test]
fn a_card_is_taken_as_text_never_as_a_local_path() {
    let (home, config) = vdir();
    let secret = home.path().join("secret.vcf");
    fs::write(&secret, CARD.replace("ada", "eve")).unwrap();

    let list = json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list"});
    let path = call(
        2,
        "card_create",
        json!({"addressbook": "personal", "vcard": secret}),
    );
    let text = CARD.replace("ada", "bob").replace("Ada", "Bob");
    let card = call(
        3,
        "card_create",
        json!({"addressbook": "personal", "vcard": text}),
    );
    let answers = serve(&home, &config, &["--allow-write"], &[list, path, card]);

    assert!(tool_names(&answers[1]).contains(&"card_delete"));
    assert_eq!(answers[2]["result"]["isError"], true, "{}", answers[2]);
    assert_ne!(answers[3]["result"]["isError"], true, "{}", answers[3]);

    let book = fs::read_dir(home.path().join("contacts/personal")).unwrap();
    let cards: Vec<String> = book
        .map(|entry| fs::read_to_string(entry.unwrap().path()).unwrap())
        .collect();
    assert_eq!(cards.len(), 2, "{cards:?}");
    assert!(cards.iter().any(|card| card.contains("FN:Bob")));
    assert!(!cards.iter().any(|card| card.contains("eve")));
}
