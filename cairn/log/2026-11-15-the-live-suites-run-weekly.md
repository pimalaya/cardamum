---
cairn: log
change: the-live-suites-run-weekly
date: 2026-11-15
---

# The live suites run weekly

Line N0 of the CLI alignment round. Capability **packaging** moved.

- **.github/workflows/tests.yml**: the `live` job runs on `workflow_dispatch` and on a weekly `schedule` (`0 3 * * 1`), never on push; `tests` and `features` stay on push. Matrix, secrets and steps unchanged.
- No live test creates a provider resource (the `credentialed::` tests list only), so no long-lived address book or contact group is needed.
- CONTRIBUTING.md, cairn/spec/testing/README.md and packaging.md ("Live provider suites") say the suites run weekly and by hand (`gh workflow run tests.yml`), a release running them by hand on the commit it tags.
