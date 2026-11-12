---
cairn: change
id: the-shared-repl-and-mcp
status: landed
created: 2026-11-12
---

# The shared REPL and MCP server

## Why

Ninth change of the-family-standards (owner, 2026-11-12: REPL and MCP, last before the release polish). Himalaya and Ortie run a session over one account and an MCP server on pimalaya-cli's shared core (himalaya the-shared-repl-and-mcp); cardamum had neither.

## What

- src/session.rs: a `Session` carries the account the global flags name and keeps the unified `AddressbookClient` across commands, built on first use and dropped when a command fails. Every command runs over it (`Command::execute(printer, session)`); the unified commands borrow the kept client, the protocol commands build theirs each time.
- `cardamum repl`: pimalaya-cli's `Repl` runs the full `Command` tree over the session, `-b/--backend` fixed, `--json` per line. It refuses by name what cannot finish inside it: `repl` and `mcp`, the composer (`-i` on `card build`, `card create` and `card update`: its editor would take the terminal the session reads, and pimalaya-cli installs no Ctrl-C handler), and a command reading stdin, refused with the new `stdin-taken` code through `pimalaya_cli::session::check_stdin` (`card create -`, the protocol bodies given as `-`). The composer also refuses itself inside any session.
- `cardamum mcp`, copied from himalaya/src/mcp.rs (`Grammar`, `mcp::serve`, `CapturePrinter`, `ToolResult::printed`, `exit_on_failure`): tools `account_capability`, `addressbook_list`, `card_list` and `card_read`, and under `--allow-write` `addressbook_create|update|delete` and `card_create|update|delete`, deletes and updates marked `destructiveHint`. A tool the account's backend does not serve is not listed. No tool takes the composer flags, and a vCard source is taken as text, never as a local path. `openWorldHint` is false on every tool: no cardamum write reaches anyone beyond the account (the family rule, owner 2026-11-12).
- tests/repl.rs and tests/mcp.rs drive the binary over a scratch vdir.

## Scope / non-goals

The protocol commands keep no client across a session. No tool for `card build` or the protocol commands.
