---
cairn: log
change: people-projection-moves-to-io-gpeople
landed: 2026-10-01
---

# The People person projection moves to io-gpeople

The Google People counterpart of [graph-projection-moves-to-io-msgraph](./2026-10-01-graph-projection-moves-to-io-msgraph.md), for the same reason: Neverest will sync Google contacts through the native API and needs the projection without depending on Cardamum.

It moved into io-gpeople behind its new `vcard` feature, as methods on `GpeoplePerson` (`to_vcard`, `from_vcard`, `changed_fields`, `unremovable_properties`), with its 9 tests, which pass there unchanged. `READ_FIELDS` became `GPEOPLE_PERSON_VCARD_FIELDS`, the stash key `GPEOPLE_PERSON_STASH_KEY` with its `cardamum.vcard` value kept, and `person_id` became `GpeoplePerson::id`, outside the feature since it has nothing to do with vCard. The minted `X-GOOGLE-*` lines are now properties pushed into the card rather than hand-escaped strings, through `VcardProp::text`.

`src/project.rs`, the helpers both projections shared, is gone: they became vcard-rs API in 0.5.1. It landed on io-gpeople 0.4.1.

Capability moved: **projection** (the People link now points at io-gpeople). No behaviour change.
