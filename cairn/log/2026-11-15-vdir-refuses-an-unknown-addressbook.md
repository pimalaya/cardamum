---
cairn: log
change: vdir-refuses-an-unknown-addressbook
date: 2026-11-15
---

# vdir refuses an unknown addressbook

A vdir addressbook id that is not one directory name names no addressbook: every vdir command naming one fails `addressbook-not-found`, exit 1, removing nothing, where `addressbook delete -k ..` answered `{"id":"..","queued":false}` and removed everything in the directory holding the home, the home and its siblings included. Line C18 of the CLI alignment round, the twin of calendula's 2fc464f. Capabilities **backends** and **commands** moved.

- **src/vdir/client.rs**: `is_dir_name` (one normal path component that is the whole id: not empty, `.` nor `..`, no separator, a trailing one included) and `collection_dir` (such an id naming a directory under the home); `VdirClient::collection_path` refuses another with ``Collection `<id>` not found``, which `vdir delete`, `vdir rename` and every `vdir item` command call.
- **src/vdir/backend.rs**: `addressbook_path` refuses it with ``Addressbook `<id>` not found`` in `update_addressbook` (now before its listing), `delete_addressbook`, `list_cards`, `get_card`, `create_card`, `update_card` and `delete_card`; `create_addressbook` refuses a name that is not one directory name, uncoded.
- **src/vdir/create.rs**: `vdir create` refuses such a name, uncoded. io-vdir 0.3 already refuses the new name of `vdir rename` (`is_name`) and a card id holding a separator (`is_stem`).
- **src/account/context.rs**: the shared `-k` resolver's empty-id refusal is coded `addressbook-not-found`, its message kept.
- **pimdir**: no gap, its ids being store rows and `known_collection` guarding every card read and write.
- **Evidence**: tests/addressbook_not_found.rs runs the binary over a home at <T>/outer/home beside <T>/outer/keep holding a file. On the old code, `an_addressbook_reaching_out_of_the_home_is_not_found` failed on `addressbook delete -k ..` (exit 0, the sibling gone), `the_vdir_commands_refuse_an_unknown_collection` on `vdir delete ""` (exit 0, the home gone) and `a_new_name_reaching_out_of_the_home_is_refused` on `addressbook create ..` (exit 0, a `displayname` written beside the home); the unknown-id and the existing-addressbook tests passed on both.

CHANGELOG (Fixed). Spec: backends ("vdir refuses an unknown addressbook", added), commands ("An empty addressbook id is rejected", modified).
