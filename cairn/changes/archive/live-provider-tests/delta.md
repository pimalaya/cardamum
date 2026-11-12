---
cairn: delta
change: live-provider-tests
---

# Delta

## ADDED Requirements

### Requirement: Live provider suites
(packaging) tests/google.rs, tests/microsoft.rs and tests/fastmail.rs SHALL run the built binary against Google (People, CardDAV), Microsoft Graph and Fastmail (JMAP, CardDAV), ignored unless asked: `anonymous::` with a made-up token on every push, `credentialed::` read-only on the test accounts with tokens the suite mints, in the cardamum repository once its secrets exist. A token SHALL reach the binary through its environment only, and no assertion SHALL print a card's value.

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
