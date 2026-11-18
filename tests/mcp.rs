//! # MCP
//!
//! The built binary's `mcp` server over a vdir account: it introduces
//! itself, lists the reading tools alone unless `--allow-write`, answers a
//! call with the payload its output schema describes, refuses a message
//! that is no request and goes on, takes a card as text, never as a local
//! path nor stdin, and fails an unknown argument, a missing one or a value
//! the grammar refuses as `usage-invalid`.

#![cfg(feature = "vdir")]

use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Output, Stdio},
};

use serde_json::{Value, json};
use tempfile::TempDir;

const CARD: &str = "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:ada\r\nFN:Ada\r\nEND:VCARD\r\n";

/// The tools listed without a flag, in their listing order.
const READS: &[&str] = &[
    "account_capability",
    "addressbook_list",
    "card_list",
    "card_read",
    "card_search",
];

/// A home holding a vdir addressbook `personal` with one card, and the
/// configuration of an account reading it.
fn account() -> TempDir {
    let home = TempDir::new().unwrap();
    let book = home.path().join("contacts").join("personal");
    fs::create_dir_all(&book).unwrap();
    fs::write(book.join("ada.vcf"), CARD).unwrap();

    let contacts = home.path().join("contacts");
    let toml = format!(
        "[accounts.example]\ndefault = true\nvdir.root = {:?}\n",
        contacts.display().to_string(),
    );
    fs::write(home.path().join("config.toml"), toml).unwrap();

    home
}

/// Runs a server with `flags` over the account in `home`, feeding it
/// `input` as it is.
fn spawn(home: &Path, flags: &[&str], input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cardamum"))
        .arg("-c")
        .arg(home.join("config.toml"))
        .arg("mcp")
        .args(flags)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home)
        .env_remove("XDG_CONFIG_DIRS")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}

/// The answers of a server fed `input`, one JSON value per line.
fn serve_bytes(home: &Path, flags: &[&str], input: &[u8]) -> Vec<Value> {
    let output = spawn(home, flags, input);
    let stdout = String::from_utf8(output.stdout).unwrap();

    stdout
        .lines()
        .map(|line| serde_json::from_str(line).unwrap_or_else(|err| panic!("{err}: {line}")))
        .collect()
}

/// The answers of a server sent `requests` after the handshake, the
/// handshake's own answer left out.
fn serve(home: &Path, flags: &[&str], requests: &[Value]) -> Vec<Value> {
    let initialize = json!({
        "jsonrpc": "2.0", "id": 0, "method": "initialize",
        "params": {"protocolVersion": "2025-06-18", "capabilities": {},
                   "clientInfo": {"name": "test", "version": "0"}},
    });
    let initialized = json!({"jsonrpc": "2.0", "method": "notifications/initialized"});

    let mut input = Vec::new();
    for message in [&initialize, &initialized].into_iter().chain(requests) {
        writeln!(input, "{message}").unwrap();
    }

    let mut answers = serve_bytes(home, flags, &input);
    answers.remove(0);
    answers
}

fn call(id: u64, name: &str, arguments: Value) -> Value {
    json!({
        "jsonrpc": "2.0", "id": id, "method": "tools/call",
        "params": {"name": name, "arguments": arguments},
    })
}

fn list(id: u64) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "method": "tools/list"})
}

fn tools(answer: &Value) -> &Vec<Value> {
    answer["result"]["tools"].as_array().unwrap()
}

fn names(answer: &Value) -> Vec<&str> {
    tools(answer)
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect()
}

fn tool<'a>(answer: &'a Value, name: &str) -> &'a Value {
    tools(answer)
        .iter()
        .find(|tool| tool["name"] == name)
        .unwrap_or_else(|| panic!("{name} not listed"))
}

fn properties(tool: &Value) -> Vec<&str> {
    let properties = tool["inputSchema"]["properties"].as_object().unwrap();
    properties.keys().map(String::as_str).collect()
}

/// The failure object a failed call answers, from its text content.
fn failure(answer: &Value) -> Value {
    let result = &answer["result"];
    assert_eq!(result["isError"], true, "{result}");
    let text = result["content"][0]["text"].as_str().unwrap();
    serde_json::from_str(text).unwrap()
}

#[test]
fn initialize_tells_the_agent_how_to_use_the_tools() {
    let home = account();
    let initialize = json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {"protocolVersion": "2025-06-18"},
    });
    let answers = serve_bytes(home.path(), &[], format!("{initialize}\n").as_bytes());

    let result = &answers[0]["result"];
    assert_eq!(result["serverInfo"]["name"], "cardamum");
    assert_eq!(result["protocolVersion"], "2025-06-18");

    let instructions = result["instructions"].as_str().unwrap_or_default();
    for word in ["addressbook_list", "card_list", "card_add", "--allow-write"] {
        assert!(instructions.contains(word), "{word}: {instructions}");
    }
}

#[test]
fn reading_tools_only_without_flags() {
    let home = account();
    let answers = serve(home.path(), &[], &[list(1)]);

    assert_eq!(names(&answers[0]), READS);

    for tool in tools(&answers[0]) {
        assert_eq!(tool["inputSchema"]["additionalProperties"], false, "{tool}");
        assert_eq!(tool["annotations"]["readOnlyHint"], true, "{tool}");
        assert_eq!(tool["annotations"]["destructiveHint"], false, "{tool}");
        assert_eq!(tool["annotations"]["openWorldHint"], false, "{tool}");

        if let Some(schema) = tool.get("outputSchema") {
            assert_eq!(schema["type"], "object", "{tool}");
        }
    }
}

#[test]
fn writes_are_listed_under_allow_write() {
    let home = account();
    let answers = serve(home.path(), &["--allow-write"], &[list(1)]);
    let listed = &answers[0];

    let mut expected = READS.to_vec();
    expected.extend([
        "addressbook_create",
        "addressbook_update",
        "addressbook_delete",
        "card_add",
        "card_update",
        "card_delete",
    ]);
    let mut names = names(listed);
    names.sort_unstable();
    expected.sort_unstable();
    assert_eq!(names, expected);

    for tool in tools(listed) {
        let properties = properties(tool);
        for gated in ["interactive", "composer"] {
            assert!(!properties.contains(&gated), "{gated}: {tool}");
        }
        assert_eq!(tool["annotations"]["openWorldHint"], false, "{tool}");
    }

    let add = tool(listed, "card_add");
    assert!(properties(add).contains(&"full-name"), "{add}");
    assert!(properties(add).contains(&"vcard"), "{add}");
    assert_eq!(add["annotations"]["readOnlyHint"], false);
    assert_eq!(add["annotations"]["destructiveHint"], false);

    for name in ["addressbook_update", "addressbook_delete", "card_update"] {
        assert_eq!(tool(listed, name)["annotations"]["destructiveHint"], true);
    }

    // NOTE: the positional keeps the name the v0.3 flag gave the property,
    // required on a delete and optional on an update.
    let delete = tool(listed, "addressbook_delete");
    assert_eq!(properties(delete), ["addressbook"], "{delete}");
    assert_eq!(delete["inputSchema"]["required"], json!(["addressbook"]));
    let update = tool(listed, "addressbook_update");
    assert!(properties(update).contains(&"addressbook"), "{update}");
    assert_eq!(update["inputSchema"]["required"], json!([]), "{update}");
}

#[test]
fn a_message_that_is_no_request_is_refused_and_the_server_goes_on() {
    let home = account();
    let mut input = Vec::new();
    input.extend_from_slice(br#"[{"jsonrpc":"2.0","id":1,"method":"ping"}]"#);
    input.extend_from_slice(b"\n{\"jsonrpc\":\"2.0\",\"id\":\"\xff\",\"method\":\"ping\"}\n");
    input.extend_from_slice(br#"{"id":3,"method":"ping"}"#);
    input.extend_from_slice(b"\n");
    input.extend_from_slice(br#"{"jsonrpc":"2.0","id":4,"method":"ping"}"#);
    input.extend_from_slice(b"\n");

    let answers = serve_bytes(home.path(), &[], &input);
    assert_eq!(answers.len(), 4, "{answers:#?}");

    assert_eq!(answers[0]["error"]["code"], -32600, "{}", answers[0]);
    assert_eq!(answers[0]["id"], Value::Null);
    assert_eq!(answers[1]["error"]["code"], -32700, "{}", answers[1]);
    assert_eq!(answers[1]["id"], Value::Null);
    assert_eq!(answers[2]["error"]["code"], -32600, "{}", answers[2]);
    assert_eq!(answers[2]["id"], 3);
    assert_eq!(
        answers[3],
        json!({ "jsonrpc": "2.0", "id": 4, "result": {} })
    );
}

/// What a tool answers matches the `outputSchema` it was listed with:
/// every required key there, no key it does not declare.
#[test]
fn a_call_answers_the_payload_its_output_schema_describes() {
    let home = account();
    let answers = serve(
        home.path(),
        &[],
        &[
            list(1),
            call(2, "addressbook_list", json!({})),
            call(3, "card_list", json!({"addressbook": "personal"})),
            call(4, "card_search", json!({"query": ["ada"]})),
        ],
    );

    for (answer, name, key) in [
        (&answers[1], "addressbook_list", "addressbooks"),
        (&answers[2], "card_list", "cards"),
        (&answers[3], "card_search", "cards"),
    ] {
        let result = &answer["result"];
        assert_ne!(result["isError"], true, "{answer}");

        let content = result["structuredContent"].as_object().unwrap();
        let schema = &tool(&answers[0], name)["outputSchema"];
        let declared = schema["properties"].as_object().unwrap();

        for required in schema["required"].as_array().unwrap() {
            let required = required.as_str().unwrap();
            assert!(content.contains_key(required), "{name}: {required}");
        }
        for key in content.keys() {
            assert!(declared.contains_key(key), "{name}: {key}");
        }

        assert!(!content[key].as_array().unwrap().is_empty(), "{answer}");
    }

    let cards = &answers[2]["result"]["structuredContent"];
    assert_eq!(cards["cards"][0]["id"], "ada");
    assert!(cards["nextPage"].is_null());
    assert_eq!(cards["errors"], json!([]));
}

/// `account_capability` answers one row per capability, each naming the
/// backend serving it, as its output schema describes.
#[test]
fn account_capability_answers_one_row_per_capability() {
    let home = account();
    let answers = serve(
        home.path(),
        &[],
        &[list(1), call(2, "account_capability", json!({}))],
    );

    let tool = tool(&answers[0], "account_capability");
    let row = &tool["outputSchema"]["$defs"]["AccountCapability"];
    let required = row["required"].as_array().unwrap();
    assert!(required.contains(&json!("backend")), "{row}");

    let result = &answers[1]["result"];
    assert_ne!(result["isError"], true, "{result}");

    let output = &result["structuredContent"];
    assert_eq!(output["account"], "example", "{output}");
    assert!(output.get("backend").is_none(), "{output}");

    let rows = output["capabilities"].as_array().unwrap();
    let row = rows.iter().find(|row| row["name"] == "card.list").unwrap();
    assert_eq!(row["backend"], "vdir");
    assert_eq!(row["level"], "full");
}

/// A listing naming a card it could not read answers its payload, the
/// card in its `errors`, as a tool error.
#[test]
fn a_listing_with_errors_answers_its_payload_as_a_tool_error() {
    let home = account();
    let book = home.path().join("contacts").join("personal");
    fs::write(book.join("broken.vcf"), "not a card").unwrap();

    let answers = serve(
        home.path(),
        &[],
        &[call(1, "card_list", json!({"addressbook": "personal"}))],
    );

    let result = &answers[0]["result"];
    assert_eq!(result["isError"], true, "{result}");

    let content = &result["structuredContent"];
    assert_eq!(content["cards"].as_array().unwrap().len(), 1, "{result}");
    assert_eq!(content["cards"][0]["id"], "ada", "{result}");
    assert_eq!(content["errors"][0]["id"], "broken", "{result}");
    assert_eq!(content["errors"][0]["code"], "card-unreadable", "{result}");
}

/// An argument the tool does not declare is a tool error naming the ones
/// it does; a tool not listed is a protocol error.
#[test]
fn an_unknown_argument_is_a_tool_error() {
    let home = account();
    let answers = serve(
        home.path(),
        &[],
        &[
            call(1, "card_list", json!({"query": "ada"})),
            call(2, "card_add", json!({"full-name": "Nope"})),
        ],
    );

    let report = failure(&answers[0]);
    let error = report["error"].as_str().unwrap();
    assert!(error.contains("Unknown argument `query`"), "{error}");
    assert!(error.contains("`addressbook`"), "{error}");
    assert_eq!(report["code"], "usage-invalid");

    assert_eq!(answers[1]["error"]["code"], -32602, "{}", answers[1]);
}

/// A value the tool's schema admits but its command's grammar refuses
/// fails the call with the failure object coded `usage-invalid`, naming
/// the argument as the tool does rather than as clap's usage text.
#[test]
fn a_value_the_grammar_refuses_is_a_usage_failure() {
    let home = account();
    let answers = serve(
        home.path(),
        &[],
        &[call(
            1,
            "card_list",
            json!({"addressbook": "personal", "page-size": -1}),
        )],
    );

    let report = failure(&answers[0]);
    assert_eq!(report["code"], "usage-invalid", "{report}");
    let error = report["error"].as_str().unwrap();
    let named = error.starts_with("Invalid value `-1` for `page-size`: ");
    assert!(named, "{report}");
}

/// A call missing a required argument names it as the tool's input does,
/// never clap's usage text.
#[test]
fn a_missing_argument_is_named_as_the_tool_takes_it() {
    let home = account();
    let answers = serve(
        home.path(),
        &["--allow-write"],
        &[
            call(1, "card_read", json!({"addressbook": "personal"})),
            call(2, "addressbook_delete", json!({})),
        ],
    );

    let report = failure(&answers[0]);
    assert_eq!(report["code"], "usage-invalid", "{report}");
    assert_eq!(report["error"], "Missing argument `card-id`", "{report}");

    let report = failure(&answers[1]);
    assert_eq!(report["code"], "usage-invalid", "{report}");
    assert_eq!(
        report["error"], "Missing argument `addressbook`",
        "{report}"
    );
}

#[test]
fn a_card_is_taken_as_text_never_as_a_local_path() {
    let home = account();
    let secret = home.path().join("secret.vcf");
    fs::write(&secret, CARD.replace("ada", "eve")).unwrap();
    let bob = CARD.replace("ada", "bob").replace("Ada", "Bob");

    let answers = serve(
        home.path(),
        &["--allow-write"],
        &[
            call(
                1,
                "card_add",
                json!({"addressbook": "personal", "vcard": secret}),
            ),
            call(
                2,
                "card_add",
                json!({"addressbook": "personal", "vcard": "-"}),
            ),
            call(
                3,
                "card_add",
                json!({"addressbook": "personal", "vcard": bob}),
            ),
        ],
    );

    let report = failure(&answers[0]);
    let error = report["error"].as_str().unwrap();
    assert!(error.contains("neither a readable file"), "{error}");
    assert_eq!(failure(&answers[1])["code"], "stdin-taken");
    assert_ne!(answers[2]["result"]["isError"], true, "{}", answers[2]);

    let book = fs::read_dir(home.path().join("contacts/personal")).unwrap();
    let cards: Vec<String> = book
        .map(|entry| fs::read_to_string(entry.unwrap().path()).unwrap())
        .collect();
    assert_eq!(cards.len(), 2, "{cards:?}");
    assert!(cards.iter().any(|card| card.contains("FN:Bob")));
    assert!(!cards.iter().any(|card| card.contains("eve")));
}

/// `addressbook_delete` takes the address book as the property its v0.3
/// flag named, which the command line now takes as its positional.
#[test]
fn an_addressbook_is_deleted_by_its_property() {
    let home = account();
    fs::create_dir(home.path().join("contacts/work")).unwrap();

    let answers = serve(
        home.path(),
        &["--allow-write"],
        &[call(
            1,
            "addressbook_delete",
            json!({"addressbook": "work"}),
        )],
    );

    let result = &answers[0]["result"];
    assert_ne!(result["isError"], true, "{result}");
    assert_eq!(result["structuredContent"]["id"], "work", "{result}");
    assert!(!home.path().join("contacts/work").exists());
    assert!(home.path().join("contacts/personal").exists());
}
