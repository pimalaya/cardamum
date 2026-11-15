---
cairn: log
change: a-raw-request-stays-on-its-origin
date: 2026-11-15
---

# A raw request stays on its origin

`msgraph request` and `gpeople request` refuse a full URL whose origin is not the API base's (`https://graph.microsoft.com`, `https://people.googleapis.com`) before anything is sent, so the account's bearer token reaches no other host: `Cannot send a request to <origin>: it is off the API origin <base>, which alone receives the account's token`, uncoded. The check runs on the resolved URL, catching an absolute URL that `Url::join` would put over the base (`HTTPS://attacker.example/…`, `ftp://…`). A path, a next link and Graph's `beta` are sent as before. `jmap request` takes no URL, posting to the session's `apiUrl` alone; its help says so. Capability **commands** moved.

Tests: `a_url_off_the_graph_origin_is_refused` and `a_url_off_the_people_origin_is_refused` (another host, plain HTTP, another port, a lookalike host or another googleapis.com host, an upper-case scheme, `ftp:`), each seen failing with the check taken out; `a_url_on_the_graph_origin_resolves` and `a_url_on_the_people_origin_resolves`.
