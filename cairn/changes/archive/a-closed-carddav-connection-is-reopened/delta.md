---
cairn: delta
change: a-closed-carddav-connection-is-reopened
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

### Requirement: A closed CardDAV connection is reopened
(backends) Every CardDAV request SHALL run through io-webdav's client: one of its methods, or `WebdavClientStd::run` for a request it has no method for (the `carddav report raw` REPORT, the `PROPFIND` `account check` sends a `carddav.home`). A request meeting a connection the server closed before answering it (a keep-alive timed out, `Connection: close`) is then sent again once on a fresh connection, as io-webdav sends its own. No command SHALL write to or read from the client's stream by hand.

#### Scenario: A raw report after the server closed the connection
- GIVEN a CardDAV home set `/dav/` whose server closed the connection the client holds
- WHEN `cardamum --json carddav report raw "My Book" '<x/>'` runs
- THEN the REPORT is sent on that connection, then the same request on a fresh one, whose multistatus prints, where the command failed with `reached unexpected EOF`

## MODIFIED Requirements

### Requirement: A raw CardDAV report addresses and counts as io-webdav does
(commands) `carddav report raw` SHALL send its REPORT to the path io-webdav builds for the address book (`rfc4918::collection_path`): an absolute id as given, a relative one joined to the home set and percent-encoded, a slash closing it; the id is refused first, as every CardDAV command refuses one ("A CardDAV id addresses its own resource" in backends). It SHALL run it through io-webdav's client ("A closed CardDAV connection is reopened" in backends), a refusal carrying the code any CardDAV refusal carries ("A server's refusal carries its code" in backends). Its `dataBytes` SHALL be the size of the row's whole `address-data` text, the final CRLF included, the size of the body `carddav get` returns.

## REMOVED Requirements
