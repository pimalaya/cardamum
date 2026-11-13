---
cairn: change
id: the-exit-contract
status: landed
created: 2026-11-13
---

# The exit contract

## Why

pimalaya-cli changed the family exit contract on 2026-11-13: 2 is clap's own status for a command line the grammar refuses, so a partial outcome sharing it left a script unable to tell a typo from a failed backend. A partial outcome moves to 3, and a refused command line under `--json` prints a failure object coded `usage-invalid` on stdout, where clap printed text on stderr that a JSON caller could not read. No cardamum release carried the old numbering.

## What

- pimalaya-cli from the local tree (`[patch.crates-io]` path `../cli`) until it is pushed.
- src/main.rs parses through `pimalaya_cli::clap::parse::<Cli>()`.
- src/mcp.rs turns a tool call the grammar refuses into `usage_error`, so it carries `usage-invalid`.
- `account check` with a failed backend exits 3 (`PartialOutcome::default()`); its doc comment, unit test and the live suites' `json_check_failure` follow.
- tests/exit.rs runs the built binary: `--json card list --nope` prints the `usage-invalid` object and exits 2, without `--json` clap's text on stderr, a failed `account check` exits 3. tests/mcp.rs checks a call with `page-size: -1`.
- Spec, CHANGELOG, MIGRATION.md.
