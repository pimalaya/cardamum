---
cairn: tasks
change: pimdir-engine-merge
---

# Tasks

- [x] `Cargo.toml`: `pimdir` = io-pimdir alone, io-replica and humantime gone, `[patch.crates-io]` on ../io-pimdir
- [x] Port the imports onto `io_pimdir::client::{reader, producer, blobs}`, `object`, `placement`, `codec`, `summary`
- [x] `add` and `update` name the body alone; `enqueue` takes no timestamp
- [x] The listing reads `list_summaries` and previews from `PimdirSummary::Contact`
- [x] Delete the JSON meta reader and the timestamp helper
- [x] Tests: the duplicate-`UID` projection on the typed summary, the preview escaping
- [x] build (`pimdir`, default, `rustls-ring,pimdir,vdir`), test, clippy, fmt
- [x] Fold the delta, log, archive
