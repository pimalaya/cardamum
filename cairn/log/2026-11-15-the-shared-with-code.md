---
cairn: log
change: the-shared-with-code
date: 2026-11-15
---

# The shared with_code

pimalaya-cli is pinned to 51ce8b17 (round 2 item 13), which shares `pimalaya_cli::error::with_code(err, code)`, the helper cardamum carried as `error::with_code` and its private `chained`. Both are gone: `Command::execute`, the MCP failure, `unreadable_card` and the backend tests call `with_code(err, error::code)`, and `card_not_found` calls it with a closure coding `card-not-found`. Capabilities moved: none.

The behaviour is unchanged. The shared helper returns `err` untouched when its chain already carries a cardamum code, as the local one did, and also when it carries `usage-invalid` or a `PartialOutcome`, which no typed backend error accompanies; otherwise it codes with what `error::code` finds, which past a coded error is the typed-error walk the local one called. `card_not_found` wraps a fresh error from a typed HTTP failure, which carries no code. Evidence: every test asserting a code passes unchanged (the carddav, gpeople and msgraph refusal tables, `a_refused_write_prints_its_code_in_its_failure_object`, tests/not_found.rs, the `card-unreadable` entries of tests/exit.rs and tests/listing.rs); the deleted `a_code_added_keeps_the_messages_of_the_chain` tested the helper's mechanics, now pimalaya-cli's. The Cargo.lock diff is pimalaya-cli's source line alone.
