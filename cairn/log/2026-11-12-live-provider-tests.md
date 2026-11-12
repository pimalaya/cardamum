---
cairn: log
change: live-provider-tests
date: 2026-11-12
---

# Live provider suites

tests/google.rs, tests/microsoft.rs and tests/fastmail.rs run the binary against the providers, written by a subagent in a scratch copy and merged. On 2026-11-12, at the code of the-shared-repl-and-mcp: the 10 anonymous tests pass (HTTP 401 on People, Google CardDAV's authenticated `.well-known` probe, Graph, Fastmail JMAP and CardDAV), and the 9 credentialed ones pass on the Google Workspace (People and CardDAV under the `contacts` and `carddav` scopes, no grant needed) and the Microsoft tenant (Graph, client credentials), each `card list -s 5` returning five cards and a `nextPage`. The subagent saw the same results on the code before capability-shared-api. The CI `live` job runs them.

Noted, not changed: `account check` exits 1 with the bare failure object when a backend fails where Himalaya exits 2 with its report; every unified addressbook reads `default: false` outside pimdir; Google CardDAV's discovery failure carries Google's raw JSON body.

Capabilities moved: packaging (added "Live provider suites").
