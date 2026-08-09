---
cairn: change
id: vdir-write-guards
status: landed
created: 2026-08-09
---

# Delta

## ADDED Requirements

## MODIFIED Requirements

### Requirement: A shared update never creates
The shared `card update` and `addressbook update` SHALL fail when the target does not exist, rather than creating it. On a backend whose write verb is create-or-replace (CardDAV `PUT`, a filesystem write), the adapter SHALL establish that the target exists before writing: CardDAV reads the card's current version and guards the write with the ETag it returns, and vdir reads the item. The wildcard `If-Match: *` SHALL NOT be used for this: RFC 9110 §13.1.1 defines it, but iCloud rejects it with `412` even for a resource that exists. A protocol-specific command SHALL keep its verb's native semantics instead: `carddav put` stays create-or-replace, gated only by the preconditions the caller passes.

### Requirement: Clearing an optional field removes it
A shared update SHALL distinguish "leave this field untouched" from "clear this field", and SHALL carry the difference all the way to the storage: on CardDAV a cleared property leaves as a `DAV:remove` instruction (RFC 4918 §9.2), never as an omitted `DAV:set`, and on vdir a cleared property has its metadata file removed rather than left in place. A backend that cannot express removal SHALL report that rather than accept the request and drop it. The adapter SHALL forward the diff as such, never reading the current state to merge unchanged fields by hand, except where the storage takes a whole desired state rather than a patch.

### Requirement: An unsupported precondition is refused, not dropped
A backend with no entity-tag concept SHALL refuse `--if-match` rather than accept and ignore it, as msgraph and jmap do. Silently dropping a guard the caller asked for is the one outcome that reads as protection and is none.

## REMOVED Requirements
