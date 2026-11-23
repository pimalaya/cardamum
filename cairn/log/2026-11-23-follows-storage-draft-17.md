---
cairn: log
change: follows-storage-draft-17
date: 2026-11-23
---

# cardamum follows storage-draft-17

Cardamum moves from io-pimdir 0.19.1 (storage-draft-15) to io-pimdir 0.20.0 (storage-draft-17, search-draft-17, sync-draft-15). The dependency is still pinned to the release candidate 17d4ea8 by a `[patch.crates-io]` git pin, with the requirement already at `version = "0.20.0"`, until io-pimdir 0.20.0 is published. Plan and evidence: cairn/changes/follows-storage-draft-17 (proposal.md, tasks.md, inventory.md, journal.md).

- **Packaging**: the network backends and vdir take io-pimdir's `contact` feature alone (the card matcher), `pimdir` takes `client`, `contact` and `search`, and the tests take `indexer` as a dev-dependency. Only the pimdir backend links SQLite now, and the binary no longer links ical-rs or mime-rs. url moves to 2.5.9 (RUSTSEC-2026-0592).
- **Stores**: a store an earlier draft wrote is refused `store-version`, never migrated. The B5 binary on a draft-15 store answers, exit 1: "Pimdir store is storage-draft-15, written by an earlier sync engine, and cardamum reads storage-draft-17 alone: a store is never migrated, so sync it with the engine that wrote it until nothing is queued, then move its directory aside and sync a new store". The README gains the start-over steps.
- **Search**: `--sort` of `card search` and `pimdir search` shows `title` and `relevance`, with `name` kept as a hidden alias of `title`. The reserved forms `~`, `/` and `{` are refused `query-refused` (`reserved-form`), and quoted they are searched. `label:` and `property:` are answered on pimdir and refused `query-unsupported` elsewhere. `mid:`, instants and the units `min`, `h` and `mo` are accepted. A refused query's text says `Error:` once (TRI-7).
- **Queue**: a `set-property` row lists, shows and cancels as any row. An undecodable pending row is an `action-malformed` item error named by its handle, and the other rows of its address book are listed (io-pimdir N1). `pimdir status` no longer fails on such a row.
- **Speed**: `pimdir search --page <n>` walks the earlier pages on one opened index without reading their cards. `card list` reads no more rows than its page needs, and reads the rows it skips, like those `pimdir status` counts, without summaries.
- **Evidence**: gate G (fmt, clippy `-D warnings` over the 11 CI sets, `cargo test --all-features` 375 passed, 0 failed, 19 ignored, the vdir and carddav CI steps 157 passed), `cargo deny check` clean, and `nix build .#default` at 7f16b27 and 17d4ea8. The final tables are in inventory.md I11: every median is at or under 0.19.1's, and every page-1 read at 100k cards is under 50 ms (`pimdir search alice` 399 to 30 ms, `pimdir status` 321 to 132 ms, `card list --page 200` 38 to 16 ms).
- **Cross-app**: on neverest's draft-17 store, cardamum lists, searches and reads the card neverest synced from a local Radicale. A `card add` on a copy of that store queues a row `pimdir queue list` shows.

Capabilities moved: commands (sort, reserved forms, store terms, queue rows), backends (store version, `action-malformed`), packaging (SQLite on pimdir alone).
