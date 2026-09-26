---
cairn: log
change: pimdir-engine-merge
landed: 2026-09-03
---

# pimdir: the backend runs on the merged io-pimdir

io-replica is retired and its engine lives in io-pimdir, one crate for both parts of the standard. The backend was ported onto it through a Cargo path patch, io-pimdir being unreleased with the merge; the `pimdir` feature pulls io-pimdir alone now, io-replica and humantime gone from the manifest.

The port deleted more than it wrote. The `v: 1` JSON meta is gone from the store, replaced by one typed summary table per kind, so the `summary_of` reader that parsed it back and the `PimdirCardMeta` it produced went, and the listing scans through `list_summaries`, the read that joins the contact row onto each item. The preview renders `PimdirSummary::Contact`: `UID`, `FN` and every canonical `EMAIL`, the unescaped name escaped back so a name carrying a comma stays one content line. `PimdirAction::Add` and `Update` carry no summary any more, the owner deriving it from the body when it applies the action, so a staged write names the body alone; `enqueue` takes no timestamp either. `create_card` still reports the link id, now the one `io_pimdir::summary::contact::derive` yields for the bytes it staged, the same function the owner runs.

A store an earlier draft wrote is refused with `PimdirError::Stale` rather than reconciled, naming the table it lacks. The backend reports it as the open failure and the changelog says what to do: delete the store and let the sync recreate it.

Verified with 73 tests passing, clippy clean over all features and targets, and builds of the default set, `pimdir` with `rustls-ring`, and `rustls-ring,pimdir,vdir`. No live account was touched.

The [backends](../spec/backends.md) capability moved: one requirement added (the draft refusal), three modified (the local backends, the cache preview, the derivations).
