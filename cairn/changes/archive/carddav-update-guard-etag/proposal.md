---
cairn: change
id: carddav-update-guard-etag
status: landed
created: 2026-08-09
---

# Guard a CardDAV update with the card's ETag, not with the wildcard

The [carddav-write-guards](../carddav-write-guards/proposal.md) change stopped `card update` from creating a card under an unknown id by sending `If-Match: *`, which RFC 9110 §13.1.1 defines as "the resource must exist". Fastmail honours it. The 2026-08-09 iCloud run found that iCloud does not: it answers `412 Persist Error` to a wildcard `If-Match` even when the resource is right there, so every `card update` on iCloud now fails.

Isolated against the live server: an update carrying an explicit correct ETag succeeds, a raw `carddav put --if-match '*'` on an existing card fails with 412, and the same put unguarded succeeds. The wildcard is the only thing iCloud refuses.

The guard is still needed there: a plain PUT to an unknown id does create a card (verified with a fresh UID), which is the defect the guard exists to prevent.

## What changes

The carddav `update_card` reads the card's current version first and guards the write with the ETag it gets back, instead of the wildcard. Every server that implements ETags at all implements this form, and both Fastmail and iCloud were verified with it.

The preflight read also improves the failure it was introduced for: an unknown id now fails as a plain "not found" rather than as a bare `412` with an empty body.

An explicit `--if-match <etag>` keeps its meaning and skips the preflight, and the raw `carddav put` is untouched.

## Cost

One extra request per update, and an update now fails if the card changed between the read and the write, where before it was last-write-wins. Both are acceptable: the JMAP backend already reads the base card before updating, and a write that lands on top of a version the user never saw is exactly what a precondition is for.
