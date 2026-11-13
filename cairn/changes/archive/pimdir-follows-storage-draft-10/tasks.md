---
cairn: tasks
change: pimdir-follows-storage-draft-10
---

# Tasks

- [x] Build against io-pimdir 0.11: `Remove`'s new members, `enqueue_with_body`, the per-kind coverage
- [x] `delete_card` states `if_match`, the test of a delete parked after a sync replaced its body
- [x] `delete_card` enqueues without a separate `check`, logging the partial supports the enqueue answers
- [x] A test holding that an update and a delete over a conflicted card park
- [x] `missing` and `unparseable` in the coverage, its line, the JSON docs and tests; `complete` from `is_complete` alone
- [x] `lock-replaced` in `ErrorCode` and `of_pimdir`, with its fixture
- [x] The version fixtures of the tests name the draft-09 store and index
- [x] Spec (backends, commands), README, MIGRATION.md, CHANGELOG, log
- [ ] The end-to-end run on a store a draft-10 neverest synced, and the network suites: the owner's, after the stores are reinstalled
