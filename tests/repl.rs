//! # REPL
//!
//! The built binary's `repl` over a vdir account, fed a piped session:
//! every line answers once and the session survives its failures, a
//! line's `--json`, a `#` comment, a session flag and the help it asks for
//! are answered as pimalaya-cli's shared loop says, what cannot run inside
//! a session (a session, the composer, stdin) is refused by name while the
//! session goes on, and a version line prints the binary's version.

#![cfg(feature = "vdir")]

use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::Path,
    process::{Command, Stdio},
};

use serde_json::Value;
use tempfile::TempDir;

const CARD: &str = "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:ada\r\nFN:Ada\r\nEND:VCARD\r\n";

/// A home holding a vdir addressbook `personal` with the card `ada`, and
/// the configuration of an account reading it, `personal` its default.
fn account() -> TempDir {
    let home = TempDir::new().unwrap();
    let book = home.path().join("contacts").join("personal");
    fs::create_dir_all(&book).unwrap();
    fs::write(book.join("ada.vcf"), CARD).unwrap();

    let contacts = home.path().join("contacts");
    let toml = format!(
        "[accounts.example]\ndefault = true\naddressbook.default = \"personal\"\n\
         vdir.home-dir = {:?}\n",
        contacts.display().to_string(),
    );
    fs::write(home.path().join("config.toml"), toml).unwrap();

    home
}

/// The binary started with `flags` and `repl` over the account in `home`,
/// its three streams piped.
fn spawn(home: &Path, flags: &[&str]) -> std::process::Child {
    Command::new(env!("CARGO_BIN_EXE_cardamum"))
        .arg("-c")
        .arg(home.join("config.toml"))
        .args(flags)
        .arg("repl")
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home)
        .env_remove("XDG_CONFIG_DIRS")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

/// Runs a piped session started with `flags` over `lines` in `home`, and
/// returns its stdout and stderr.
fn run(home: &Path, flags: &[&str], lines: &str) -> (String, String) {
    let mut child = spawn(home, flags);

    child
        .stdin
        .take()
        .unwrap()
        .write_all(lines.as_bytes())
        .unwrap();

    let output = child.wait_with_output().unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(output.status.success(), "{stderr}");

    (String::from_utf8(output.stdout).unwrap(), stderr)
}

/// The JSON values of a session's stdout, one per line.
fn values(stdout: &str) -> Vec<Value> {
    stdout
        .lines()
        .map(|line| serde_json::from_str(line).unwrap_or_else(|err| panic!("{err}: {line}")))
        .collect()
}

/// Runs a piped `--json` session over `lines`, one JSON value per answer.
fn session(home: &Path, lines: &str) -> Vec<Value> {
    values(&run(home, &["--json"], lines).0)
}

#[test]
fn a_session_answers_each_line_and_survives_its_failures() {
    let home = account();
    let answers = session(
        home.path(),
        "addressbook list\n\
         nope\n\
         card read nope\n\
         card list \"unclosed\n\
         card list\n\
         quit\n\
         addressbook list\n",
    );

    assert_eq!(answers.len(), 5, "{answers:#?}");
    assert_eq!(answers[0]["addressbooks"][0]["id"], "personal");
    assert!(answers[1]["error"].is_string(), "{}", answers[1]);
    assert_eq!(answers[1]["code"], "usage-invalid", "{}", answers[1]);
    assert_eq!(answers[2]["code"], "card-not-found", "{}", answers[2]);
    assert!(answers[3]["error"].is_string(), "{}", answers[3]);
    assert_eq!(answers[4]["cards"][0]["id"], "ada");
}

/// A listing exiting 3 printed its result over a client that answered, so
/// the session keeps that client: the next line runs on it even once the
/// configuration it was built from is gone, where a client built again
/// would fail.
#[test]
fn a_partial_outcome_keeps_the_session_client() {
    let home = account();
    let book = home.path().join("contacts").join("personal");
    fs::write(book.join("broken.vcf"), "not a card").unwrap();

    let mut child = spawn(home.path(), &["--json"]);
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    stdin.write_all(b"card list\n").unwrap();
    let mut first = String::new();
    stdout.read_line(&mut first).unwrap();
    let first: Value = serde_json::from_str(&first).unwrap();
    assert_eq!(first["errors"][0]["id"], "broken", "{first}");

    fs::remove_file(home.path().join("config.toml")).unwrap();

    stdin.write_all(b"card list\n").unwrap();
    drop(stdin);

    let mut rest = String::new();
    stdout.read_to_string(&mut rest).unwrap();
    let answers = values(&rest);
    let status = child.wait().unwrap();
    assert!(status.success(), "{rest}");

    assert_eq!(answers.len(), 1, "{answers:#?}");
    assert_eq!(answers[0]["cards"][0]["id"], "ada", "{}", answers[0]);
    assert_eq!(answers[0]["errors"][0]["id"], "broken", "{}", answers[0]);
}

#[test]
fn a_line_json_flag_switches_that_line_alone() {
    let home = account();
    let (stdout, _) = run(
        home.path(),
        &[],
        "card list --json\n\
         addressbook list\n",
    );
    let mut lines = stdout.lines();

    let json: Value = serde_json::from_str(lines.next().unwrap()).unwrap();
    assert_eq!(json["cards"][0]["id"], "ada", "{stdout}");

    let text: Vec<&str> = lines.collect();
    assert!(
        text.iter().any(|line| line.starts_with("personal")),
        "{stdout}"
    );
    assert!(!text.iter().any(|line| line.starts_with('{')), "{stdout}");
}

#[test]
fn a_session_flag_is_refused_by_name() {
    let home = account();
    let answers = session(
        home.path(),
        "card list -a other\n\
         card list -b vdir\n\
         card list --backend=carddav\n\
         card list -c other.toml\n",
    );

    assert_eq!(answers.len(), 4, "{answers:#?}");

    for (answer, flag) in answers.iter().zip(["-a", "-b", "--backend", "-c"]) {
        let message =
            format!("Cannot set `{flag}` inside a session, start a new session with it instead");
        assert_eq!(answer["error"], message, "{answer}");
        assert_eq!(answer["code"], "usage-invalid", "{answer}");
    }
}

#[test]
fn a_comment_line_is_skipped() {
    let home = account();
    let answers = session(
        home.path(),
        "# addressbook list\n\
         \x20  # card delete ada\n\
         card list # every one\n",
    );

    assert_eq!(answers.len(), 1, "{answers:#?}");
    assert_eq!(answers[0]["cards"][0]["id"], "ada");
}

#[test]
fn help_goes_to_stderr_in_a_json_session() {
    let home = account();
    let (stdout, stderr) = run(
        home.path(),
        &["--json"],
        "help\n\
         card list --help\n\
         card list\n",
    );

    let answers = values(&stdout);
    assert_eq!(answers.len(), 1, "{stdout}");
    assert_eq!(answers[0]["cards"][0]["id"], "ada");

    let usage = "Usage: cardamum [OPTIONS] <COMMAND>";
    assert!(stderr.contains(usage), "{stderr}");
    let usage = "Usage: cardamum card list [OPTIONS]";
    assert!(stderr.contains(usage), "{stderr}");
}

#[test]
fn what_cannot_run_in_a_session_is_refused_and_the_session_goes_on() {
    let home = account();
    let answers = session(
        home.path(),
        "repl\n\
         mcp --allow-write\n\
         card add -i\n\
         card update -i --composer true ada\n\
         card build -i\n\
         card create -\n\
         card update ada -\n\
         card list\n",
    );

    assert_eq!(answers.len(), 8, "{answers:#?}");

    for answer in &answers[..2] {
        let error = answer["error"].as_str().unwrap_or_default();
        assert_eq!(error, "Cannot start a session inside a session", "{answer}");
        assert_eq!(answer["code"], "usage-invalid", "{answer}");
    }

    for answer in &answers[2..5] {
        let error = answer["error"].as_str().unwrap_or_default();
        assert!(error.contains("composer (-i)"), "{answer}");
    }

    assert_eq!(answers[5]["code"], "stdin-taken", "{}", answers[5]);
    assert_eq!(answers[6]["code"], "stdin-taken", "{}", answers[6]);

    let cards = answers[7]["cards"].as_array().unwrap();
    assert_eq!(cards.len(), 1, "{cards:?}");
    assert_eq!(cards[0]["id"], "ada");
}

/// A version line prints the version the command line prints, `-V` the
/// short one and `--version` the long one, after a command as alone, and
/// the session goes on.
#[test]
fn a_version_line_prints_the_version() {
    let home = account();
    let (stdout, _) = run(
        home.path(),
        &[],
        "-V\n--version\ncard list --version\ncard list --json\n",
    );

    let version = concat!("cardamum ", env!("CARGO_PKG_VERSION"));
    let long = concat!("cardamum v", env!("CARGO_PKG_VERSION"), " ");
    let mut lines = stdout.lines();

    assert_eq!(lines.next(), Some(version), "{stdout}");
    let line = lines.next().unwrap_or_default();
    assert!(line.starts_with(long), "{stdout}");
    // NOTE: clap names the subcommand, as the command line does.
    let sub = lines.find(|line| line.starts_with("cardamum-card-list v"));
    assert!(sub.is_some(), "{stdout}");

    let last = values(stdout.lines().last().unwrap_or_default());
    assert_eq!(last[0]["cards"][0]["id"], "ada", "{stdout}");
}
