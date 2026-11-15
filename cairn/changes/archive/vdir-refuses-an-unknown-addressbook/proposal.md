---
cairn: change
id: vdir-refuses-an-unknown-addressbook
status: landed
created: 2026-11-15
---

# vdir refuses an unknown addressbook

> Line C18 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), a data-loss defect found by calendula's K2 and reproduced on cardamum the same day.

## Why

The vdir backend refused an addressbook only when the path it built was no directory (src/vdir/backend.rs, `addressbook_path`; src/vdir/client.rs, `VdirClient::collection_path`), so an id that is not one directory name reached out of the home. On a scratch vdir whose home is <T>/outer/home, holding `friends`, beside <T>/outer/keep, 2026-11-15:

- `cardamum --json addressbook delete -k ..` answered `{"id":"..","queued":false}`, exit 0, and removed everything in <T>/outer: the home, every addressbook in it and the sibling directory;
- `addressbook delete -k ../keep` removed the sibling, `-k friends/..` and `-k .` everything in the home (the latter then failing uncoded);
- `vdir delete ""` removed the home, `vdir delete ..` everything in the directory holding it;
- `card list -k .` listed the home as an addressbook;
- `addressbook create ..` and `vdir create ..` wrote a `displayname` file into the directory holding the home, `addressbook create ../keep` into the sibling.

calendula fixed the same hole (2fc464f, "vdir refuses an unknown calendar"); himalaya's Maildir and mbox refuse `..`, `../keep` and `""` already.

## What

- `vdir::client::is_dir_name`: an id is one directory name when it is one normal path component and the whole of the id, so neither empty, `.` nor `..`, and holding no separator, a trailing one included. `collection_dir` resolves such an id to its directory when the home holds one.
- `VdirBackend::addressbook_path` refuses any other id with `addressbook-not-found`, ``Addressbook `<id>` not found``, before anything is read or written: `addressbook update` (now before the listing it reads), `addressbook delete`, and every card list, read, create, update and delete.
- `VdirClient::collection_path` refuses it the same way with ``Collection `<id>` not found``: `vdir delete`, `vdir rename` and every `vdir item` command (list, get, create, update, delete, copy and move, both collections).
- A new name: `addressbook create` and `vdir create` refuse a name that is not one directory name, uncoded, before anything is written; io-vdir 0.3 already refuses the new name of `vdir rename` (`is_name`: empty, a leading dot, a NUL or a separator) and a card id holding a separator (`is_stem`).
- The shared `-k` resolver's refusal of an empty id is coded `addressbook-not-found`, as it names none, its message kept: the unified commands taking `-k` refuse `""` there, before any backend.
- pimdir has no such gap: its addressbook ids are rows of the store, never paths, and `known_collection` refuses an unknown one on every card read and write.
- Spec: backends ("vdir refuses an unknown addressbook", added), commands ("An empty addressbook id is rejected", modified).
