---
cairn: tasks
change: vdir-refuses-an-unknown-addressbook
---

# Tasks

- [x] Tests through the binary (tests/addressbook_not_found.rs) on a scratch vdir whose home sits in a directory beside a sibling holding a file: `..`, `../keep`, `friends/..`, `.`, `friends/` and `""` remove nothing and fail `addressbook-not-found` on `addressbook delete` and `card list`, `card read`, `card delete`; every unified command naming `nope` fails coded; `vdir delete`, `vdir rename` and `vdir item` refuse an unknown or outreaching collection; `addressbook create`, `vdir create` and a `vdir rename` new name reaching out are refused; an addressbook the home holds still lists and deletes alone. Three of the five failed on the old code (`addressbook delete -k ..` exit 0 removing the sibling, `vdir delete ""` exit 0 removing the home, `addressbook create ..` exit 0 writing out of the home).
- [x] `is_dir_name` and `collection_dir` in src/vdir/client.rs, `VdirBackend::addressbook_path` and `VdirClient::collection_path` on top of them; `update_addressbook` checks the id first.
- [x] `addressbook create` (vdir) and `vdir create` refuse a name that is not one directory name; `vdir rename`'s new name and card ids checked in io-vdir 0.3.
- [x] The shared `-k` resolver's empty id coded `addressbook-not-found`.
- [x] pimdir checked: ids are store rows, `known_collection` guards every card read and write.
- [x] CHANGELOG Fixed line.
- [x] Fold the delta into backends.md and commands.md, write the log entry, set `landed`, archive.
