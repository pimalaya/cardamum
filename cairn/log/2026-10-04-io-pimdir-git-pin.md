---
cairn: log
change: io-pimdir-git-pin
landed: 2026-10-04
---

# io-pimdir pinned with neverest, himalaya and calendula

io-pimdir moves from the 0.6 release on crates.io to the git revision `8c83c04`, the commit neverest, himalaya and calendula pin, so the four tools open a store with one io-pimdir and one schema: a store migrated by one of them (the receipts table, pimdir draft-04) is never opened by an older reader. cardamum's code is unchanged; tests run with `--no-default-features --features pimdir`, as MOA builds it.

Capabilities moved: none.
