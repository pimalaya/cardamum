---
cairn: log
change: the-kit-carries-the-shared-outputs
date: 2026-11-15
---

# The kit carries the shared outputs

cardamum moves to pimalaya-cli 0.5.1 and pimalaya-config 0.5.1 (Cargo.lock: those two crates alone) and drops its copies of what they now carry. Lines L1, L4, L5 and L7 of cli-alignment-2026-11-15 reach cardamum. Capabilities **commands**, **backends**, **repl**, **mcp** and **config** moved.

- **account list, account check**: `pimalaya_cli::account::{AccountListOutput, AccountRow, AccountCheckOutput, BackendCheck}`, each row coded through `with_code(err, error::code)`. JSON unchanged; the `account list` text is the family's (no header row, `default` on the default account, the built backends then those left out).
- **REPL**: the nested `repl`/`mcp` arm is gone, the kit refusing the line first, coded `usage-invalid`; `Command` carries the version, so `-V` and `--version` lines print it.
- **MCP**: a refused call goes through `Grammar::usage_error` (``Missing argument `card-id` ``, ``Invalid value `-1` for `page-size`: …``).
- **card build -o**: the kit's `SavedOutput`, its text `Saved <size> bytes to <path>`.
- **Configuration**: the removed tables and `carddav.discover` through `pimalaya_config::removed_key!`; the sentences keep their shape, no key moves.
- **Evidence**: tests/repl.rs (`mcp` nested coded, a version line, refused as an unknown flag before), tests/mcp.rs (the grammar's wording, a missing argument), the config test asserting each whole sentence.
