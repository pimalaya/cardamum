---
cairn: change
id: the-live-suites-run-weekly
status: landed
created: 2026-11-15
---

# The live suites run weekly

> Line N0 of the family's CLI alignment round, X10 (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15, tasks.md).

## Why

Every push to master ran the `live` job against Google, Microsoft and Fastmail. Across the family a push storm burns Google's quota, hits Graph throttling and collides with the owner's live testing, so the family moves every credentialed suite off push onto one weekly slot per repository, and onto a run by hand.

## What

- .github/workflows/tests.yml: `on:` gains `workflow_dispatch:` and `schedule:` at `0 3 * * 1` (Mondays 03:00 UTC, cardamum's slot), and the `live` job runs only on those two events. Its matrix, secrets and steps stay; the `tests` and `features` jobs stay on push.
- No test of cardamum creates a provider resource: the `credentialed::` tests of tests/google.rs and tests/microsoft.rs only list, so no long-lived address book or contact group is needed.
- CONTRIBUTING.md, cairn/spec/packaging.md ("Live provider suites") and cairn/spec/testing/README.md say the suites run weekly and by hand (`gh workflow run tests.yml`), and that a release runs them by hand on the commit it tags.
