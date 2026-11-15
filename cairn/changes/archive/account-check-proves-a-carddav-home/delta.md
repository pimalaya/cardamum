---
cairn: delta
change: account-check-proves-a-carddav-home
---

# Delta

## ADDED Requirements

### Requirement: account check proves a CardDAV home
(backends) `account check` on a `[carddav]` block naming `home` SHALL send the home one authenticated `PROPFIND` of depth 0, asking for `DAV:resourcetype`, once connected, and SHALL fail the `carddav` row when it is refused, with the message and the code the `server` route's discovery fails with on the same answer: io-webdav's `WebDAV server returned HTTP <status>`, a 401 carrying no code and a 403 `permission-denied` ("A server's refusal carries its code"). The `home` route connects without a request, so nothing else in the check proves the credential. The request SHALL be the check's alone, the other commands sending none: the first request each sends is authenticated, so a refused credential fails it, and the `home` route exists to save the discovery round trips. The `server` route sends none either, its walk proving the credential.

#### Scenario: A made-up token against a CardDAV home
- GIVEN an account whose `carddav.home` names a server answering its token 401
- WHEN `cardamum --json account check` runs
- THEN it exits 3, the `carddav` row `ok: false`, `code` `null` and its `error` naming HTTP 401, as with `carddav.server` naming the same server

## MODIFIED Requirements

## REMOVED Requirements
