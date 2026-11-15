---
cairn: tasks
change: a-raw-request-stays-on-its-origin
---

# Tasks

- [x] src/msgraph/request.rs, src/gpeople/request.rs: `resolve_url` refuses a URL off the base's origin; help text says so.
- [x] src/jmap/request.rs: help says the request goes to the session's `apiUrl` alone.
- [x] Tests `a_url_off_the_graph_origin_is_refused`, `a_url_off_the_people_origin_is_refused` (seen failing with the check taken out), and their on-origin twins.
- [x] Spec (commands), CHANGELOG, MIGRATION.
