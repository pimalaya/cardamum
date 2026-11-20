---
cairn: log
change: an-unreadable-card-read-is-coded
date: 2026-11-20
---

# An unreadable card read is coded

`card read` on a card whose vdir file or pimdir body could not be read (a blob with no read permission) failed exit 1 with `code: null` and the bare `Permission denied (os error 13)` (vdir: io-vdir's `Failed to read file <path>: ...`), naming no card, while `card list` names the same card a `card-unreadable` entry. Bug K-A of the pimdir backend test run. Capabilities **commands** and **backends** moved.

- **src/error.rs**: `card_unreadable`, cardamum's sentence ``Cannot read card `<id>` in address book `<addressbook>` `` over the backend's error, coded as it is or `card-unreadable`, as `card_not_found` words a miss.
- **src/pimdir/backend.rs**: `get_card` and `version` take a blob read failing with an I/O error through it, so `card read`, `card update` starting from the card, the MCP `card_read` (the same command) and an `update_card` or `delete_card` given `--if-match` fail coded. `update_card` and `delete_card` read the version only when `--if-match` is given, where they read it always and failed on such a body even with no guard; `check_version` takes the version asked for.
- **src/vdir/backend.rs**: `get_card` codes a read failing otherwise than not found the same way.
- **Evidence**: `an_unreadable_body_fails_its_read_coded` (src/pimdir/backend.rs) and `a_card_file_that_cannot_be_read_fails_its_read_coded` (tests/listing.rs) fail on the HEAD sources (`code` `null`, `Permission denied (os error 13)`; `Failed to read file ...`) and pass after.

CHANGELOG (Fixed). Spec: commands (the `card-unreadable` row), backends ("pimdir is a cache, not a server", one scenario). Left as it was: `pimdir search` failing whole on an unreadable hit, an io-pimdir bug fixed there.
