---
cairn: change
id: cli-alignment-leftovers
status: active
created: 2026-10-18
---

# What cardamum still owes the family alignment

## Why

The cross-repo alignment audit of 2026-10-18 (CLI_ALIGNMENT_PLAN.md, which kept the item ids of the 2026-08-28/29 plan) found these items still open in cardamum. Each was re-checked against the code on 2026-10-18.

- **C3**: a write on the pimdir backend only queues the change locally, yet the shared confirmation says "Card `<id>` successfully updated" (src/shared/card/update.rs:158), and create and delete say the same (src/shared/card/create.rs:132, src/shared/card/delete.rs:40).
- **C4**: `pimdir.account` is taken as configured or left unset (src/pimdir/client.rs:39-50). himalaya derives it from the store when one account synced it and errors by name when several did (himalaya/src/pimdir/client.rs:225).
- **B3**: the `carddav` block has no `alpn` key, the client pins `http/1.1` (src/carddav/client.rs:303), where `jmap`, `msgraph` and `gpeople` expose one.
- **G2**: each backend built at runtime gets its own `SecretResolver` (src/shared/client.rs:238-252).
- **A2**: the `[patch.crates-io]` git pins wait for releases.

pimalaya-cli will take over two local copies once its changes land: wizard-save-layer replaces `ExistingConfig`, `append_or_print` and `print_saved` (src/wizard/configure.rs:91, :220, :246), shared-account-resolution replaces `resolve_account` (src/cli.rs:174).

## What

Close each item above, one task each. C3 and C4 are pimdir behaviour, so they fold into the pimdir spec; the rest are config and dependency hygiene.
