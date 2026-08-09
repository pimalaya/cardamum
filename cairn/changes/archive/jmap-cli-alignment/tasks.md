---
cairn: tasks
change: jmap-cli-alignment
---

# Tasks

- [x] `contact-card copy`: positional repeatable ids, long-only `--from-account`, repeatable `--to-address-book`
- [x] `address-book get`: positional optional ids
- [x] Card report prints only the fields the object carries; an empty update response reports success in words
- [x] Port Himalaya's `JmapSetError` / `format_set_error` into src/jmap/error.rs and use it at every `rejected` site
- [x] Add the `vCardProps` hint naming the offending vCard properties
- [x] cargo fmt, clippy, feature matrix
- [x] Re-test live against `fastmail-jmap`, then update the reports, fold the delta, log and archive
