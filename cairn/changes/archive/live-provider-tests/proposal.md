---
cairn: change
id: live-provider-tests
status: landed
created: 2026-11-12
---

# Live provider suites

## Why

Seventh change of the-family-standards. Provider APIs drift without notice, and cardamum had no test reaching one: the manual reports under cairn/spec/testing are dated runs. Himalaya's live suites (himalaya live-provider-tests) run the built binary against the test accounts on every push.

## What

- tests/common.rs: the binary in a TempDir home, its token read through `["printenv", "CARDAMUM_TEST_TOKEN"]`, the accounts written from config.sample.toml's sections, and shape assertions that name a faulty member, never a card's value.
- tests/google.rs (People and Google CardDAV), tests/microsoft.rs (Graph) and tests/fastmail.rs (JMAP and CardDAV), every test ignored unless `--ignored`: `anonymous::` runs a made-up token, touching no account (`account check` and `addressbook list` fail with HTTP 401); `credentialed::` reads the test accounts (`account check`, `addressbook list`, `card list -s 5`) with tokens minted by io-oauth 0.5.2: a service account impersonating `google@pimalaya.org` with exactly the `contacts` and `carddav` scopes (one refused scope failing the whole request), and the Microsoft test application's client credentials, `msgraph.user-id` naming the test mailbox. Fastmail is anonymous only, as in Himalaya.
- .github/workflows/tests.yml gains Himalaya's `live` job: the anonymous tests per provider on every push, the credentialed ones in pimalaya/cardamum once its secrets exist.
- The owner grant the umbrella planned is not needed: the Google delegation already serves `contacts` and `carddav`, and the Microsoft application holds `Contacts.ReadWrite`.

## Scope / non-goals

The suites only read. Landed after capability-shared-api and the-shared-repl-and-mcp, out of the umbrella's order: they ran on the code before and after both, every served verb printing the same.
