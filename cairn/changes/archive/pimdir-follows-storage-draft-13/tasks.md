---
cairn: tasks
change: pimdir-follows-storage-draft-13
---

# Tasks

- [x] Cargo.toml `io-pimdir = "0.14.0"` (through a git pin at d184622 until it published), no `[patch.crates-io]` left, Cargo.lock moved for io-pimdir alone.
- [x] src/pimdir/queue/show.rs: `PimdirQueueRefusal`, `refusals` on the output, the text lines, `on card <seq>`.
- [x] src/pimdir/status.rs and `PimdirBackend::status`: `PimdirRefusedStatus`, `refused`.
- [x] Tests: `an_applied_row_prints_the_refusals_on_its_card`; `a_refused_push_shows_on_its_row_and_in_the_status` (a drained update, a `RefusePlacement` written as an engine would, read back by `queue show` and `status`), seen failing with the status read taken out; the store-version fixture moved to `storage-draft-11`, asserting the rebuild.
- [x] Spec folded (commands, backends), CHANGELOG, MIGRATION, README.
