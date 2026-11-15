---
cairn: log
change: the-session-tests-follow-calendula
date: 2026-11-15
---

# The session tests follow calendula

tests/repl.rs and tests/mcp.rs reach calendula's coverage, and src/mcp.rs gains its two classification tests. Task C16 of cli-alignment-2026-11-15 (finding F45). Capabilities moved: mcp (the spec states the two checks); no behaviour changed.

- **tests/repl.rs** (3 tests, now 8): every line answered and failures survived (`usage-invalid`, `card-not-found`, an unclosed quote), a partial outcome keeping the session client, a line's `--json`, the session flags refused by name and coded, a `#` comment skipped, help on stderr in a `--json` session, the nested session, the composer and stdin refused, the version lines.
- **tests/mcp.rs** (7 tests, now 12): `initialize` and its instructions, the reading tools and their annotations, the writes under `--allow-write` and the `addressbook_*` schemas, a message that is no request refused while the server goes on, the payload matching its `outputSchema`, an unknown argument, a missing argument (`card-id`, `addressbook`), a path and stdin refused as a vCard source, an address book deleted by its property. The listing-with-errors test takes an unparseable card where it removed a file's read permission, so it runs on every platform.
- **src/mcp.rs**: `every_unified_leaf_is_classified` (`card build` excluded) and `the_instructions_name_existing_tools`, as calendula's.
