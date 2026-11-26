---
cairn: log
change: pimdir-1-0-codes-relayed
landed: 2026-11-26
---

# cardamum relays io-pimdir 1.0.0's `source-empty`

io-pimdir 1.0.0 added two codes: `source-empty` (`PimdirError::SourceEmpty`, a store bound to a source of an empty id) and `target-unsynced` (`PimdirMutateError::Unsynced`, a copy or a move into a collection no source syncs). cardamum's relay table (src/error.rs `PIMDIR_CODES`) is Himalaya's set, and Himalaya relays `source-empty`, so cardamum now does too.

- **Code**: `ErrorCode::SourceEmpty`, serialised `source-empty`, in the `cardamum-error` schema. As 2026-11-26-follows-pimdir-v1 says, no cardamum path raises it: it opens no source; the entry keeps the one vocabulary every pimdir program reads.
- **Not relayed**: `target-unsynced`. It is `PimdirMutateError::code`, raised only on the owner's run, which cardamum never drives; a copy into an unsynced target reaches a producer as a parked queue row, not as an error. Himalaya leaves it out for the same reason.
- **Tests**: `a_relayed_pimdir_code_keeps_io_pimdirs_spelling` covers the spelling; `a_pimdir_failure_prints_the_code_io_pimdir_names` gains `PimdirError::SourceEmpty`.
- **Docs**: CHANGELOG 1.0.0 (the pimdir writes and queue codes), the code table of commands.md, the relayed list of backends.md.

Capabilities moved: commands (one code), backends (the relayed list).
