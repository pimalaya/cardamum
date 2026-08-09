---
cairn: change
id: carddav-write-guards
status: landed
created: 2026-08-09
---

# Delta

## ADDED Requirements

### Requirement: A shared update never creates
The shared `card update` and `addressbook update` SHALL fail when the target does not exist, rather than creating it. On a backend whose write verb is create-or-replace (CardDAV `PUT`), the client SHALL add the existence precondition the protocol offers (`If-Match: *`, RFC 9110 §13.1.1) whenever the caller supplied no stricter one. A protocol-specific command SHALL keep its verb's native semantics instead: `carddav put` stays create-or-replace, gated only by the preconditions the caller passes.

### Requirement: Clearing an optional field removes it
A shared update SHALL distinguish "leave this field untouched" from "clear this field", and SHALL carry the difference all the way to the wire: on CardDAV a cleared property leaves as a `<D:remove>` instruction (RFC 4918 §9.2), never as an omitted `<D:set>`. A backend that cannot express removal SHALL report that rather than accept the request and drop it.

## MODIFIED Requirements

## REMOVED Requirements
