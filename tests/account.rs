//! # Account and backend resolution
//!
//! The built binary's failure object for an account it cannot take, and
//! for a backend the build leaves out: its block loads, so one file
//! serves every build, but a command reaching it fails coded
//! `backend-not-built`, naming the cargo feature. The second half runs on
//! a reduced build, such as `cargo test --no-default-features --features
//! rustls-ring,vdir --test account`.

use std::{
    fs,
    path::Path,
    process::{Command, Output, Stdio},
};

use serde_json::Value;
use tempfile::TempDir;

/// Runs the binary under `--json` with `args` over the configuration
/// `toml`, in a home of its own, its stdin closed.
fn run(home: &Path, toml: &str, args: &[&str]) -> Output {
    let config = home.join("config.toml");
    fs::write(&config, toml).unwrap();

    Command::new(env!("CARGO_BIN_EXE_cardamum"))
        .args(["--json", "-c", config.to_str().unwrap()])
        .args(args)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home)
        .env_remove("XDG_CONFIG_DIRS")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

/// What `output` printed, once it exited with `status`.
fn printed(output: &Output, status: i32) -> Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(status), "{stdout}");
    serde_json::from_str(&stdout).unwrap()
}

#[test]
fn an_unknown_account_name_is_coded_account_missing() {
    let home = TempDir::new().unwrap();
    let toml = "[accounts.a]\ndefault = true\n";

    let output = run(home.path(), toml, &["-a", "nope", "account", "capability"]);
    let json = printed(&output, 1);
    assert_eq!(json["code"], "account-missing", "{json}");
}

#[test]
fn the_name_default_is_a_name_like_any_other() {
    let home = TempDir::new().unwrap();
    let toml = "[accounts.a]\ndefault = true\n";

    let output = run(
        home.path(),
        toml,
        &["-a", "default", "account", "capability"],
    );
    let json = printed(&output, 1);
    let error = json["error"].as_str().unwrap();
    assert_eq!(json["code"], "account-missing", "{json}");
    assert!(error.contains("omit `-a`"), "{error}");
}

#[test]
fn no_account_marked_default_is_coded_account_default_missing() {
    let home = TempDir::new().unwrap();
    let toml = "[accounts.a]\n[accounts.b]\n";

    let output = run(home.path(), toml, &["account", "capability"]);
    let json = printed(&output, 1);
    assert_eq!(json["code"], "account-default-missing", "{json}");
}

#[test]
fn several_accounts_marked_default_are_coded_account_default_several() {
    let home = TempDir::new().unwrap();
    let toml = "[accounts.a]\ndefault = true\n[accounts.b]\ndefault = true\n";

    let output = run(home.path(), toml, &["account", "capability"]);
    let json = printed(&output, 1);
    assert_eq!(json["code"], "account-default-several", "{json}");
}

/// An account over the People backend alone, which the build leaves out.
#[cfg(not(feature = "gpeople"))]
const GPEOPLE: &str = "[accounts.a]\ndefault = true\n\
                       gpeople.auth.token.raw = \"token\"\n\
                       proxy.url = \"socks5h://localhost:1080\"\n";

/// Asserts `json` is the failure of reaching the People backend.
#[cfg(not(feature = "gpeople"))]
fn assert_gpeople_not_built(json: &Value) {
    let error = json["error"].as_str().unwrap();
    assert_eq!(json["code"], "backend-not-built", "{json}");
    assert!(error.contains("`--features gpeople`"), "{error}");
}

#[cfg(not(feature = "gpeople"))]
#[test]
fn a_unified_command_over_a_backend_left_out_is_backend_not_built() {
    let home = TempDir::new().unwrap();

    let output = run(home.path(), GPEOPLE, &["addressbook", "list"]);
    assert_gpeople_not_built(&printed(&output, 1));

    let output = run(home.path(), GPEOPLE, &["card", "list", "-k", "book"]);
    assert_gpeople_not_built(&printed(&output, 1));
}

#[cfg(not(feature = "gpeople"))]
#[test]
fn account_capability_over_a_backend_left_out_is_backend_not_built() {
    let home = TempDir::new().unwrap();

    let output = run(home.path(), GPEOPLE, &["account", "capability"]);
    assert_gpeople_not_built(&printed(&output, 1));
}

#[cfg(not(feature = "gpeople"))]
#[test]
fn account_check_reports_a_backend_left_out_as_failed() {
    let home = TempDir::new().unwrap();

    let output = run(home.path(), GPEOPLE, &["account", "check"]);
    let json = printed(&output, 3);
    assert_eq!(json["backends"][0]["backend"], "gpeople", "{json}");
    assert_eq!(json["backends"][0]["ok"], false, "{json}");
    assert_eq!(json["backends"][0]["code"], "backend-not-built", "{json}");
}

#[cfg(not(feature = "gpeople"))]
#[test]
fn account_list_names_a_backend_left_out() {
    let home = TempDir::new().unwrap();

    let output = run(home.path(), GPEOPLE, &["account", "list"]);
    let json = printed(&output, 0);
    assert_eq!(json["accounts"][0]["backends"][0], "gpeople", "{json}");
    assert_eq!(json["accounts"][0]["notBuilt"][0], "gpeople", "{json}");
}

#[cfg(not(feature = "gpeople"))]
#[test]
fn a_backend_flag_naming_a_backend_left_out_names_its_feature() {
    let home = TempDir::new().unwrap();

    let args = ["--backend", "gpeople", "account", "capability"];
    let output = run(home.path(), GPEOPLE, &args);
    let json = printed(&output, 2);
    let error = json["error"].as_str().unwrap();
    assert_eq!(json["code"], "usage-invalid", "{json}");
    assert!(error.contains("`--features gpeople`"), "{error}");
}

/// The backend `auto` resolves to is the same in every build: a build
/// leaving the first one out fails rather than take the next, which
/// `--backend` still picks.
#[cfg(all(not(feature = "vdir"), feature = "carddav"))]
#[test]
fn auto_never_skips_a_backend_left_out() {
    let home = TempDir::new().unwrap();
    let toml = "[accounts.a]\ndefault = true\nvdir.home-dir = \"/nonexistent\"\n\
                carddav.server = \"https://example.org\"\n\
                carddav.auth = \"none\"\n";

    let output = run(home.path(), toml, &["account", "capability"]);
    let json = printed(&output, 1);
    assert_eq!(json["code"], "backend-not-built", "{json}");

    let args = ["--backend", "carddav", "account", "capability"];
    let output = run(home.path(), toml, &args);
    let json = printed(&output, 0);
    assert_eq!(json["capabilities"][0]["backend"], "carddav", "{json}");
}
