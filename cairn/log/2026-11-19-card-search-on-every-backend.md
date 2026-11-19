---
cairn: log
change: card-search
date: 2026-11-19
---

# Card search built on every backend

- `card search`, its MCP tool `card_search` and its schema are built in with any backend, where they needed `vdir` or `pimdir`: each network feature (`carddav`, `jmap`, `msgraph`, `gpeople`) pulls io-pimdir 0.17.0 under `search` and ariadne, no rusqlite. A CardDAV-only build gains ical-rs and mime-rs through io-pimdir's `search`; no lighter feature split is asked of io-pimdir for now.
- The matching path (src/shared/card/search.rs `matched_page`) answers for any backend not pimdir, over `list_addressbooks` and `list_cards`. Test `a_graph_search_matches_every_folder_depth`: over a scripted Graph session, `elodie` finds `Élodie Martin` two folders deep, which failed before Graph listed folders at every depth.
- `card.search` stays `none` on CardDAV, JMAP, Microsoft Graph and Google People: its `partial` declaration waits for a search over the largest test address book to be measured on the live accounts, then flips four rows of src/capability.rs.

Spec: commands (the card search requirements), mcp (`card_search` in every build).
