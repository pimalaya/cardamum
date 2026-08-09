---
cairn: change
id: carddav-update-guard-etag
status: landed
created: 2026-08-09
---

# Delta

## ADDED Requirements

## MODIFIED Requirements

### Requirement: A shared update never creates
The shared `card update` and `addressbook update` SHALL fail when the target does not exist, rather than creating it. On a backend whose write verb is create-or-replace (CardDAV `PUT`), the adapter SHALL read the target's current version first and guard the write with the ETag it returns, so that an unknown id fails on the read as "not found". The wildcard `If-Match: *` SHALL NOT be used for this: RFC 9110 §13.1.1 defines it, but iCloud rejects it with `412` even for a resource that exists. A protocol-specific command SHALL keep its verb's native semantics instead: `carddav put` stays create-or-replace, gated only by the preconditions the caller passes.

## REMOVED Requirements
