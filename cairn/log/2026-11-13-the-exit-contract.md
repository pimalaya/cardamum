---
cairn: log
change: the-exit-contract
date: 2026-11-13
---

# The exit contract

cardamum follows pimalaya-cli's exit contract of 2026-11-13: 0 success, 1 failure object, 2 a command line the grammar refuses, 3 a partial outcome. Capabilities **commands**, **mcp** and **repl** moved.

- **Cargo.toml, Cargo.lock**: pimalaya-cli from the local tree (`path = "../cli"`), to re-pin once it is pushed.
- **src/main.rs**: `pimalaya_cli::clap::parse::<Cli>()`. `cardamum --json card list --nope` prints `{"code":"usage-invalid",...}` on stdout and exits 2; without `--json`, clap's text on stderr, exit 2.
- **src/mcp.rs**: a tool call the grammar refuses (`card_list` with `page-size: -1`) goes through `usage_error` and answers `usage-invalid`, where its code was `null`.
- **account check**: a failed backend exits 3 (`PartialOutcome::default()`); the doc comment, the unit test (asserting the status now) and tests/common.rs `json_check_failure` follow.
- **Tests**: tests/exit.rs (usage object and exit 2, stderr text without `--json`, `account check` exit 3) and tests/mcp.rs `a_value_the_grammar_refuses_is_a_usage_failure`.

Not changed: an argument the MCP grammar does not know (`Unknown argument ...`) is pimalaya-cli's own refusal and stays uncoded.

Spec correction: commands.md said a failure with no code prints no `code` field; it prints `"code": null`, as pimalaya-cli always did.
