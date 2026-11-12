---
cairn: log
change: the-family-standards
date: 2026-11-12
---

# Cardamum takes the family standards

Cardamum is on the standard Ortie and Himalaya share, in eleven local commits on 2026-11-12: published-dependencies (every crate from crates.io), the-api-is-unified (owner, Himalaya's rename), the-wizard-leaves, the-shared-kit (pimalaya-cli and pimalaya-config 0.4, the welcome, `-c` one path per flag, plain text, `card list` paged as Himalaya's `envelope list`, G2 and B3), the-sample-is-tested-and-shipped, dependencies-trimmed (`cargo deny` clean), every-feature-set-builds-clean (CI matrix), capability-shared-api (owner: the union, `capability-missing`, `account capability`), the-shared-repl-and-mcp, live-provider-tests (Google, Microsoft and Fastmail, no grant needed), account-check-reports-its-failure (found by the live suites) and release-polish. live-provider-tests landed after the capability and REPL changes, against the umbrella's order, and ran on the code before and after them.

Left for the owner: lifting the list header and footer into pimalaya-cli (cardamum and calendula carry copies of Himalaya's), the version bump, the release. pimalaya-cli stays pinned to git rev 0fbe5dd until 0.4.1 publishes `Records`.

Capabilities moved: none of its own; each change logged its own.
