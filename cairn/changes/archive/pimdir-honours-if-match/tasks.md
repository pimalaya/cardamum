---
cairn: tasks
change: pimdir-honours-if-match
---

# Tasks

- [x] pimdir: check `if_match` in `update_card` and `delete_card`.
- [x] Shared: `if_match` on `delete_card` and `--if-match` on `card delete`; CardDAV passes it, vdir, jmap, msgraph and gpeople refuse it.
- [x] Tests: the version follows a staged update; a stale tag refuses update and delete, queueing nothing; the right one deletes.
- [x] Fold the delta into [backends](../../spec/backends.md) and [commands](../../spec/commands.md); write the log entry.
