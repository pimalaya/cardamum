---
cairn: log
change: carddav-update-guard-etag
landed: 2026-08-09
---

# A CardDAV update is guarded by the card's ETag, not by the wildcard

The [carddav-write-guards](2026-08-09-carddav-write-guards.md) change, landed this morning, stopped `card update` creating a card under an unknown id by sending `If-Match: *` when the caller passed no ETag of their own. RFC 9110 §13.1.1 defines the wildcard as "the resource must exist" and Fastmail honours it. The same day's iCloud run found that iCloud does not: it answers `412 Persist Error` to a wildcard `If-Match` even for a card that is right there, so every `card update` on an iCloud account failed.

Isolated against the live server before changing anything: an update carrying an explicit correct ETag succeeds, a raw `carddav put --if-match '*'` on an existing card fails with 412, and the same put unguarded succeeds. The wildcard is the only thing iCloud refuses. The guard is still needed there, since a plain PUT to an unknown path does create a card, verified with a fresh UID.

The carddav adapter now reads the card's current version first and guards the write with the ETag that read returns. Every server implementing ETags at all implements that form, and both providers were re-verified with it. The preflight also improves the failure the guard exists for: an unknown id now fails as "Cannot read the current version of card `x`" with the server's 404 as its cause, instead of a bare 412 with an empty body.

The cost is one extra request per update, and an update now fails if the card changed between the read and the write where it used to be last-write-wins. Both are acceptable: the JMAP backend already reads the base card before updating, and a write landing on top of a version the user never saw is exactly what a precondition is for. An explicit `--if-match <etag>` skips the preflight and keeps its meaning, and the raw `carddav put` is untouched.

The requirement in [backends](../spec/backends.md) moved with it: "the adapter SHALL read the target's current version first and guard the write with the ETag it returns", with the wildcard explicitly ruled out.

Also corrected while here: the `valid-address-data` hint blamed the vCard version ("some, e.g. iCloud, require vCard 3.0 with an N property"). The iCloud run showed 3.0 and 4.0 are both stored verbatim and that the real constraint is `UID` plus `N`, so the hint now says "some (e.g. iCloud) also require an N property".
