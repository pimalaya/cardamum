---
cairn: tasks
change: an-update-guards-only-where-served
---

# Tasks

- [x] Regression test (src/shared/card/update.rs, `a_graph_update_without_if_match_sends_one_plain_patch`): a scripted Graph server answers the reads and the `PATCH`; `card update -k contacts AAMk --full-name "Ada Byron"` succeeds with one `PATCH` carrying no `If-Match`. It failed on the old code with "Microsoft Graph does not support If-Match guarded updates".
- [x] `AddressbookClient::serves`; `card update` drops the read version on a backend declaring `card.update.if-match` at `none`; the command's help says so.
- [x] Every backend's update and delete checked: jmap leaked the same way (the hash its read reports), vdir reads no version, carddav, gpeople and pimdir take the guard, `card delete` sends only the caller's `--if-match`.
- [x] CHANGELOG Fixed line.
- [x] Fold the delta into commands.md, write the log entry, set `landed`, archive.
