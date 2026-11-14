---
cairn: log
change: year-less-gpeople-birthday
date: 2026-11-14
---

# Year-less gpeople birthday

A vCard `BDAY:--MMDD` on the gpeople backend now becomes a People birthday `{month, day}` and reads back as `BDAY:--MMDD`, where it went to the `clientData` stash and Google never saw it ([#26](https://github.com/pimalaya/cardamum/issues/26), reported and fixed by jontxux in io-gpeople#2). No capability moved: the vCard and People mapping is io-gpeople's.

- **Cargo.toml, Cargo.lock**: io-gpeople pinned in `[patch.crates-io]` to `e507d82`, the fix on io-gpeople master after 0.7.0. Drop the pin once 0.7.1 is out.
