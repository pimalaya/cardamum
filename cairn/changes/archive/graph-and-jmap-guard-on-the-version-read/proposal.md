---
cairn: change
id: graph-and-jmap-guard-on-the-version-read
status: landed
created: 2026-11-15
---

# Graph and JMAP guard on the version read

## Why

calendula guards a Graph event update and delete with the `changeKey` it read, declared `partial` (calendula/src/msgraph/backend.rs, `CHANGE_KEY`), while cardamum refused `--if-match` on Graph and JMAP, `capability-missing`, though each reports a version on a read (the Graph `changeKey`, a hash of the JMAP ContactCard). Task C22 of cli-alignment-2026-11-15, from C21 (9377aff), which made a Graph update with no `--if-match` a plain `PATCH`.

## What

- Graph: `update_card` compares `--if-match` with the `changeKey` of the contact it reads as its delta base anyway; `delete_card` reads the contact first, only when `--if-match` is given. A version that differs, surrounding quotes ignored, fails `precondition-failed` before the write.
- JMAP: the same with the hash `card read` reports, compared with the card `update_card` reads as its patch base; `delete_card` reads it first when guarded.
- `card.update.if-match` and `card.remove.if-match` are `partial` on both, the detail saying a write landing between the read and the write is not caught.
- `card update` with no source now guards with the version it read on Graph and JMAP too, at no extra request.
