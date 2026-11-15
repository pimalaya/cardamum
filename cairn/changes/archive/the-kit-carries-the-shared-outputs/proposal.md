---
cairn: change
id: the-kit-carries-the-shared-outputs
status: landed
created: 2026-11-15
---

# The kit carries the shared outputs

## Why

pimalaya-cli 0.5.1 and pimalaya-config 0.5.1 took what the family's binaries kept copies of (cli-alignment-2026-11-15, lines L1, L4, L5 and L7 of the family plan, findings F3, F41, F44): the `account list` and `account check` outputs, the file-written receipt, the refusal of a nested session, the version line of a session, the wording of a refused MCP call, and the sentence of a removed configuration key. cardamum kept its own copy of each, its `account list` text with a header row the others lack.

## What

- Move to pimalaya-cli 0.5.1 and pimalaya-config 0.5.1 (Cargo.lock: those two crates alone).
- `account list` and `account check` print `pimalaya_cli::account::{AccountListOutput, AccountCheckOutput}`, each check row built by `BackendCheck::new` over a chain coded by cardamum's reader (`with_code`). JSON unchanged; the `account list` text is the shared one: no header row, `default` on the default account, the built backends then those left out.
- The REPL's own refusal of a `repl` or `mcp` line goes: the kit refuses it first, coded `usage-invalid`.
- The command enum carries `#[command(version, long_version = long_version!())]`, so `-V` and `--version` work inside a session.
- MCP answers a refused call through `Grammar::usage_error`, naming the argument as the tool takes it.
- `card build -o` prints `pimalaya_cli::output::SavedOutput`, its text `Saved <size> bytes to <path>`.
- The removed tables and `carddav.discover` are declared through `pimalaya_config::removed_key!`, the family's sentence; no key is renamed or removed.
