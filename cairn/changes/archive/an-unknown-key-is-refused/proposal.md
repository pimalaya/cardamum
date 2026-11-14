---
cairn: change
id: an-unknown-key-is-refused
status: landed
created: 2026-11-14
---

# An unknown key is refused

## Why

The owner decided on 2026-11-14 that a configuration key Cardamum does not know is refused at every level, as Himalaya does. Cardamum refused one in its backend and option tables, but its root and its accounts tolerated any key, for a neighbouring tool sharing the file that never existed: a typo there (`defualt = true`) loaded silently, and the tables v0.3 read and this release removed (`table`, the top-level `account`) were ignored rather than flagged.

## What

- `Config` and `AccountConfig` refuse an unknown key (`deny_unknown_fields`), as every other config table already did. No config table flattens another, so no table needs restructuring.
- The tables this release removed are refused by name, the error pointing at MIGRATION.md, through uninhabited types as Calendula does: `table` (root and account), the top-level `account`, `addressbook.list` and `card.list.table` (root and account), the last two refused as unknown keys until now.
- The block of a backend the build leaves out (`carddav`, `jmap`, `msgraph`, `gpeople`, `vdir`, `pimdir`, and `proxy` in a build with no network backend) loads unread, as before, so one file still serves a reduced build: it is a key Cardamum knows, not a typo.
- Spec: config ("An unknown key is refused" added, "One backend block per backend" and "No colour is configured" modified).
