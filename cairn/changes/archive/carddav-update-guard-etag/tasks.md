---
cairn: tasks
change: carddav-update-guard-etag
---

# Tasks

- [x] carddav `update_card` guards with the current ETag, read first, instead of `If-Match: *`
- [x] The preflight read's failure carries context naming the card
- [x] cargo fmt, clippy, feature matrix
- [x] Re-verify on iCloud and on Fastmail: update lands, unknown id fails and creates nothing, explicit `--if-match` still guards
- [x] Fold the delta into the spec, update the reports, log and archive
