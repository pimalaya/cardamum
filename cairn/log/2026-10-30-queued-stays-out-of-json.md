---
cairn: log
change: queued-stays-out-of-json
landed: 2026-10-30
---

# A queued write says so in text only

Corrects 2026-10-30-pimdir-release-alignment.md, whose "A queued write says it is queued" item added a `queued` boolean to the `card create` and `card update` JSON. Only pimdir ever set it, so it was a pimdir detail in a shared output, which the requirement "Shared outputs carry no backend details" (backends) forbids, whatever the other backends print. The field is gone, never having been released: `CardCreatedOutput` and `CardUpdatedOutput` keep `queued` under `#[serde(skip)]` for the wording alone, as calendula does. `card create`, `card update` and `card delete` on pimdir still print that the next sync carries the write. Capability **commands** moved: "A queued write says it is queued" now keeps the JSON shape every backend prints.

The `ErrorCode` variants `body-pending` and `precondition-failed`, which only pimdir raises, are now compiled with the pimdir feature alone, so each other backend built on its own passes `clippy -D warnings`.

Tests: `a_queued_write_says_it_is_queued` now asserts the exact JSON (`{"id"}`, `{"id", "keptProperties"}`) beside the queued wording, and fails with the field serialised. 73 tests pass with every feature, 41 with pimdir alone, 32 with carddav alone; clippy is clean with every feature, the default ones, and each of `carddav`, `jmap`, `msgraph`, `gpeople`, `vdir` and `pimdir` alone with `rustls-ring`.
