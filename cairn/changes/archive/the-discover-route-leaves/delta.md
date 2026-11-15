---
cairn: delta
change: the-discover-route-leaves
---

# Delta

## ADDED Requirements

### Requirement: CardDAV resolves no domain
(config) `carddav.discover` SHALL be refused by name, the error naming `carddav.server` as its replacement (Fastmail's `https://carddav.fastmail.com/dav/`), `carddav.home` on Google, and MIGRATION.md. A build without `carddav` loads the block unread.

#### Scenario: A v0.3 discover route
- GIVEN an account holding `carddav.discover = "fastmail.com"`
- WHEN a command loads the configuration
- THEN it fails naming the `carddav.discover` key as removed, `carddav.server`, `carddav.home` and MIGRATION.md

## MODIFIED Requirements

### Requirement: CardDAV home-set resolution routes
(backends) The `[carddav]` block SHALL resolve the addressbook home set through one of two routes. `home` short-circuits all discovery and is used as the home set directly. `server` connects to the given context root, then walks current-user-principal (RFC 5397) and addressbook-home-set (RFC 6352). A block naming neither SHALL fail naming both.

### Requirement: CardDAV probes a bare origin before walking
(backends) Because a bare origin is not necessarily the context root (Fastmail serves contacts under `/dav/` and 404s everything else), the `server` route SHALL probe `.well-known/carddav` and follow its redirect before the principal walk whenever the server names no path.

### Requirement: A proxy is set per account and per backend
(config) The sentence on CardDAV `discover` lookups reading the environment only leaves; the rest of the requirement is unchanged.

## REMOVED Requirements

None: the `discover` route lived inside "CardDAV home-set resolution routes", modified above.
