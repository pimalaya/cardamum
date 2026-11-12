---
cairn: log
change: capability-shared-api
date: 2026-11-12
---

# The unified API serves the union of the backends

src/capability.rs declares, for each of the six backends, the thirteen unified capabilities (`addressbook.list` to `card.remove.if-match`) at `full`, `partial`, `server` or `none`, seeded from the adapters' refusals. Every unified command checks the capabilities it and its flags reach before reading or connecting and fails with the new `capability-missing` code; `account capability` lists the resolved backend's rows without connecting; each unified leaf's `--help` ends with a generated `Backends:` footer. tests/capability.rs runs the binary over a scratch vdir: `card delete --if-match` is refused with the code and the card file is untouched, `account capability` reads `vdir`, the help names the backends. A served verb prints what it printed. 92 unit tests and the 8 first-run and 3 capability tests pass; clippy is clean on all features and on `rustls-ring` with no backend, vdir, pimdir and jmap.

Capabilities moved: commands (removed "Unified commands stay least-common-denominator"; added "Capability registry", "An unserved verb is refused before connecting", "An account lists what it serves", "The help names what each backend serves"; "A JSON error carries a stable code" gains `capability-missing`), backends ("An unsupported precondition is refused, not dropped").
