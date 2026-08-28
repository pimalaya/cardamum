---
cairn: tasks
change: pimdir-reader-producer
---

# Tasks

- [x] The client holds a `PimdirReader` with the pending overlay, and opens a `PimdirProducer` per write
- [x] The three card writes enqueue `add`, `update` and `remove` instead of staging replica mutations
- [x] `pimdir.source` leaves the config, the wizard and the sample
- [x] The derivations delegate to `io_pimdir::conventions::card`, and the body hash comes from the store
- [x] `create_addressbook` refuses, naming the reason
- [x] carddav: `supported_reports`, `CarddavCardEnumOk`, `sync_cards` options after the io-webdav bump
- [x] cargo fmt, clippy, tests, feature matrix
- [x] Read-only run against the Posteo store, reported under spec/testing
- [x] Fold the delta, log and archive
