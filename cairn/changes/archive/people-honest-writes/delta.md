---
cairn: change
id: people-honest-writes
status: landed
created: 2026-08-09
---

# Delta

## ADDED Requirements

### Requirement: A partial write says so
When a backend cannot carry out part of a write, and the protocol offers no way to insist, the command SHALL report what was left undone rather than print a plain success. On Google People an unmappable vCard property lives in a `clientData` entry the API will not clear, so an update dropping one keeps it, and the command names it.

### Requirement: A displayed count is the server's
A table SHALL NOT render an absent value as a default that reads as fact. A count the API returns only on request SHALL be requested, or left blank.

## MODIFIED Requirements

## REMOVED Requirements
