//! # Shared test helpers
//!
//! The account configuration the live suites write, the built binary run
//! against it with an access token in its environment, the provider
//! sections of config.sample.toml, and the environment the credentialed
//! suites read their secrets from.

#![allow(dead_code)]

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use serde_json::Value;

/// The variable the binary reads its access token from.
pub const TOKEN_VAR: &str = "CARDAMUM_TEST_TOKEN";

/// The token command of every account the suites write: the token
/// reaches the binary through its environment, never its arguments nor
/// its configuration.
pub const TOKEN_COMMAND: &str = r#"["printenv", "CARDAMUM_TEST_TOKEN"]"#;

/// A token no provider ever issued.
pub const MADE_UP: &str = "made-up";

/// What Cardamum says of a request a provider refuses, whichever the
/// backend.
pub const REFUSED: &str = "HTTP 401";

/// Writes a configuration in `dir` holding `name` as the default
/// account, made of `lines`, and returns its path.
pub fn write_config(dir: &Path, name: &str, lines: &str) -> PathBuf {
    let config = dir.join("config.toml");
    let account = format!("[accounts.{name}]\ndefault = true\n{lines}\n");
    fs::write(&config, account).unwrap();
    config
}

/// Runs the built binary against `config`, `token` in its environment,
/// the configuration's directory as its home and its stdin closed.
pub fn cardamum(config: &Path, token: &str, args: &[&str]) -> Output {
    let home = config.parent().unwrap();

    Command::new(env!("CARGO_BIN_EXE_cardamum"))
        .arg("-c")
        .arg(config)
        .args(args)
        .env(TOKEN_VAR, token)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home)
        .env_remove("XDG_CONFIG_DIRS")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

/// What a run shows when an assertion fails: its exit code, its stdout,
/// where failures are printed, and its stderr. The token is in neither,
/// the binary reading it from its environment only.
pub fn failure(out: &Output) -> String {
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let code = out.status.code();
    format!("exit {code:?}, stdout: {stdout}, stderr: {stderr}")
}

/// The JSON a `--json` run printed, failing unless it exited with
/// `code`.
pub fn json(out: &Output, code: i32) -> Value {
    assert_eq!(out.status.code(), Some(code), "{}", failure(out));
    serde_json::from_slice(&out.stdout).unwrap_or_else(|err| panic!("{err}: {}", failure(out)))
}

/// The failure object a `--json` run printed, failing unless it exited
/// with 1 and its message holds `refusal`.
pub fn json_failure(out: &Output, refusal: &str) -> Value {
    let error = json(out, 1);
    let message = error["error"].as_str().unwrap_or_default();
    assert!(message.contains(refusal), "{error}");
    assert!(error["sources"].is_array(), "{error}");
    error
}

/// The report a `--json` `account check` run printed, failing unless it
/// exited with 2, the partial outcome, and names `backend` as failed for
/// `refusal`.
pub fn json_check_failure(out: &Output, backend: &str, refusal: &str) -> Value {
    let report = json(out, 2);
    let backends = report["backends"].as_array().unwrap();
    let check = backends.iter().find(|check| check["backend"] == backend);
    let check = check.unwrap_or_else(|| panic!("{backend} not checked: {report}"));
    let message = check["error"].as_str().unwrap_or_default();
    assert_eq!(check["ok"], false, "{report}");
    assert!(message.contains(refusal), "{report}");
    report
}

/// Fails unless `report` is the `account check` output of one backend,
/// `backend`, that answered.
pub fn assert_check(report: &Value, backend: &str) {
    let backends = report["backends"].as_array().unwrap();
    assert_eq!(backends.len(), 1, "{report}");
    assert_eq!(backends[0]["backend"], backend, "{report}");
    assert_eq!(backends[0]["ok"], true, "{report}");
    assert!(backends[0]["error"].is_null(), "{report}");
}

/// Fails unless `list` is an `addressbook list` output whose
/// addressbooks are each of the shape the binary prints, `id` among
/// them.
pub fn assert_addressbooks(list: &Value, id: &str) {
    let books = list["addressbooks"].as_array().unwrap();

    for (i, book) in books.iter().enumerate() {
        let description = &book["description"];
        let color = &book["color"];

        assert!(book["id"].is_string(), "addressbook {i}: no string id");
        assert!(book["name"].is_string(), "addressbook {i}: no string name");
        assert!(book["default"].is_boolean(), "addressbook {i}: no default");
        assert!(
            description.is_string() || description.is_null(),
            "addressbook {i}: description neither a string nor null"
        );
        assert!(
            color.is_string() || color.is_null(),
            "addressbook {i}: color neither a string nor null"
        );
    }

    let ids: Vec<_> = books.iter().filter_map(|b| b["id"].as_str()).collect();
    assert!(ids.contains(&id), "no addressbook {id} among {ids:?}");
}

/// Fails unless `list` is a `card list` output of at most `size` cards
/// of `book`, each of the shape the binary prints. A failure names the
/// faulty member, never its value: a card is personal data.
pub fn assert_cards(list: &Value, book: &str, size: usize) {
    let cards = list["cards"].as_array().unwrap();
    let count = cards.len();
    let next = &list["nextPage"];

    assert!(count <= size, "{count} cards");
    assert!(next.is_string() || next.is_null(), "nextPage: {next}");

    for (i, card) in cards.iter().enumerate() {
        assert!(card["id"].is_string(), "card {i}: no string id");
        assert_eq!(card["addressbookId"], book, "card {i}: addressbookId");

        for key in ["etag", "fnValue", "email", "tel", "uid", "fullName"] {
            let value = &card[key];
            let valid = value.is_string() || value.is_null();
            assert!(valid, "card {i}: {key} neither a string nor null");
        }

        for key in ["emails", "phones", "organizationUnits"] {
            assert!(card[key].is_array(), "card {i}: {key} not an array");
        }
    }
}

/// The commented section `name` of config.sample.toml, uncommented,
/// each line setting a key of `overrides` set to its value instead.
pub fn sample_section(name: &str, overrides: &[(&str, &str)]) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.sample.toml");
    let sample = fs::read_to_string(path).unwrap();
    let heading = format!("# {name}");

    let mut lines = sample.lines().skip_while(|line| *line != heading).skip(2);
    let mut section = String::new();

    for line in lines.by_ref().take_while(|line| !line.is_empty()) {
        let line = line.strip_prefix('#').unwrap_or(line);
        let key = line.split(" = ").next().unwrap_or_default();

        match overrides.iter().find(|(k, _)| *k == key) {
            Some((key, value)) => section.push_str(&format!("{key} = {value}\n")),
            None => section.push_str(&format!("{line}\n")),
        }
    }

    assert!(
        !section.is_empty(),
        "no {name} section in config.sample.toml"
    );
    section
}

/// The environment variable `name`, an empty value counting as unset,
/// since a workflow maps a missing secret to an empty one.
pub fn env_var(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.is_empty())
}

/// [`env_var`] of each of `names`, failing the test with every one of
/// them unset.
pub fn require_env<const N: usize>(names: [&str; N]) -> [String; N] {
    let values = names.map(env_var);
    let unset: Vec<_> = names
        .iter()
        .zip(&values)
        .filter(|(_, value)| value.is_none())
        .map(|(name, _)| *name)
        .collect();

    assert!(
        unset.is_empty(),
        "set {} to run this test",
        unset.join(", ")
    );
    values.map(Option::unwrap)
}

/// The content of the file the `{name}_FILE` variable names, or else of
/// the variable `name`.
pub fn env_content(name: &str) -> Option<String> {
    match env_var(&format!("{name}_FILE")) {
        Some(path) => Some(fs::read_to_string(path).unwrap()),
        None => env_var(name),
    }
}
