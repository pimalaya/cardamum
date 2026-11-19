---
cairn: log
change: a-request-off-its-api-is-coded
date: 2026-11-19
---

# A request off its API is coded

`msgraph request` and `gpeople request` refuse a URL off the API origin through pimalaya-cli 0.7.0's `error::request_origin`, coded `request-origin` (exit 1), the message unchanged. `jmap request` has nothing to refuse.

Tests: `a_url_off_the_graph_origin_is_refused` and `a_url_off_the_people_origin_is_refused` assert the code, the Graph one seen failing with the old uncoded `bail!`. Capability **commands** moved (the requirement, its scenario, the code table, the failure schema line).
