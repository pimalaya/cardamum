---
cairn: delta
change: the-live-suites-run-weekly
---

# Delta

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: Live provider suites
(packaging) tests/google.rs, tests/microsoft.rs and tests/fastmail.rs SHALL run the built binary against Google (People, CardDAV), Microsoft Graph and Fastmail (JMAP, CardDAV), ignored unless asked: `anonymous::` with a made-up token, `credentialed::` read-only on the test accounts with tokens the suite mints, in the cardamum repository once its secrets exist. CI SHALL run them weekly (Mondays 03:00 UTC, the family's slot for cardamum) and by hand (`gh workflow run tests.yml`), never on a push, and a release SHALL run them by hand on the commit it tags; the offline jobs run on every push. A token SHALL reach the binary through its environment only, and no assertion SHALL print a card's value.

## REMOVED Requirements

None.
