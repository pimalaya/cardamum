---
cairn: change
id: the-last-twin-lows
status: landed
created: 2026-11-15
---

# The last twin lows

> Line X13 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15): X9c, the `vdir item get` encoding, L11, and the owner's rulings D27 and D28.

## Why

- **X9c.** A duplicate address book reads ``Addressbook `x` already exists``, where Himalaya says ``Cannot create mailbox `x`: it exists already`` and calendula is moving to the same form.
- **vdir item get.** It prints `contents` with no `encoding`, and fails uncoded with "invalid utf-8 sequence" on an item that is not UTF-8, where `carddav get`, `card read` and calendula's `vdir item get` carry `encoding` and read such bytes as base64.
- **L11.** The `pimdir` and `vdir` namespaces open with "Report on, queue into and search..." and "Manage vdir collections...", where Himalaya's and calendula's read `<API>-specific API`; `jmap` and `gpeople` read `JMAP-specific API (RFC 8620 + RFC 9610)` and `Google People API-specific API`, off the same pattern.
- **D27.** The text prints `Next page:` after the listing everywhere. cardamum builds no such line of its own: `card list` and `pimdir search` print it through pimalaya-cli's `Listing::page`, which pimalaya-cli moves (0.5.3), and the protocol listings through `Paginated`, which prints it last already. Nothing to change here until the bump.
- **D28.** `addressbook create <NAME>` cannot choose its id apart from the name, where calendula's `calendar create <NAME> [--id <ID>]` can.

## What

- src/error.rs `addressbook_duplicate`, src/vdir/client.rs `check_collection_absent`: ``Cannot create address book `<id>`: it exists already``, on vdir, `vdir create`, CardDAV, Graph and People. Code unchanged, the server's answer still in `sources`. The sentence never shipped (v0.3.0 had none), so its [Unreleased] line is reworded.
- src/vdir/item/get.rs: the item through `CardContents` (`contents`, `encoding`), its text printing bytes that are not UTF-8 as they are, as `carddav get`. Breaking as M5 was for `carddav get` (CHANGELOG and MIGRATION.md lines widened).
- src/pimdir/cli.rs, src/vdir/cli.rs, src/jmap/cli.rs, src/gpeople/cli.rs: the summaries `pimdir-specific API`, `vdir-specific API`, `JMAP-specific API`, `Google People-specific API`.
- src/shared/addressbook/create.rs, src/shared/client.rs, src/vdir/backend.rs, src/carddav/backend.rs, src/capability.rs: `--id <ID>`, reaching vdir and CardDAV; a new capability `addressbook.create.id`, `full` on both, `none` on JMAP, Graph and People (the server mints the id) and pimdir, so they refuse it before connecting. Additive.
