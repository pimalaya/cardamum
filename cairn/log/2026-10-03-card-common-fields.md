---
cairn: log
change: card-common-fields
landed: 2026-10-03
---

# A card lists and reads with its common fields

`card list` and `card read` now carry the common fields of a card in their JSON (src/shared/card/project.rs): its `UID`, names, every address and number, organization, title and note, read through vcard-rs's decoded model so folds, escapes and `tel:` URIs come back as their values. The first-value preview columns stay. A pimdir round trip, from a queued create through a drain to a read of a pending update, is tested against a real store.

Capability moved: **commands** (a card projects its common fields).
