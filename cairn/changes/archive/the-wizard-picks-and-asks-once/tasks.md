---
cairn: tasks
change: the-wizard-picks-and-asks-once
---

# Tasks

- [x] Cargo.toml: io-pim-discovery 0.11 with `autoconfig`, `pacc`, `rfc6764`, `rfc8620`, `stream`; `pimalaya-cli = { path = "../cli" }` in `[patch.crates-io]`.
- [x] wizard/search.rs: options per protocol, server and sign-in, deduplicated; the services this build compiles; the ranking into Auto; the login rule; the review wording.
- [x] wizard/review.rs: the review loop over pimalaya-cli `review::prompt`, the Change list naming a server only where two entries read the same.
- [x] wizard/server.rs: the `Server URL:` prompt, its scheme parse refusing `http://`, then Password or API token.
- [x] wizard/sign_in.rs: password and API token typed once, tested, stored; OAuth through `ortie::sign_in` or the broker list; the refusal menu; the backend config builder.
- [x] wizard/remote.rs: the remote input parse, discovery under its spinner, the nothing-discovered path, the review then sign-in loop.
- [x] wizard/secret.rs: `store`, `broker_token`, `to_secret`; drop `configure_password` and `configure_token`.
- [x] wizard/discover.rs and configure.rs: the input prompted first, the account name settled before discovery, the local flow tested at the end.
- [x] Remove wizard/{carddav,jmap,gpeople,msgraph}.rs, their prompts gone.
- [x] Tests: dedupe, ranking with and without Ortie, login rule, review sentences, Change labels, server URL parsing, remote input parse.
- [x] README, config.sample.toml, src/main.rs header, CHANGELOG.
- [x] fmt, clippy, tests, feature matrix.
- [x] Fold the delta into cairn/spec/wizard.md, log, archive.
