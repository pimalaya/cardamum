---
cairn: log
change: the-wizard-picks-and-asks-once
date: 2026-11-08
---

# The wizard picks, and asks once

The wizard now follows the Pimalaya Android app's onboarding, transposed to the CLIs from the flow the owner approved on 2026-11-08, Himalaya taking the same change in parallel. One input, discovery under a `Looking up <domain>` spinner, the best-ranked option taken as Auto, pimalaya-cli's review with one `Contacts` row in plain words, then the sign-in, typed once and tested at once.

Dependencies: io-pim-discovery 0.9 to 0.11 with `autoconfig` added (its MX provider detection; the autoconfig mechanisms themselves only run for mail services, which cardamum does not ask for), bringing psl and serde-xml-rs into Cargo.lock, and pimalaya-stream 0.3.1. pimalaya-cli is patched to the local checkout at 6d8daa7 (`[patch.crates-io] pimalaya-cli = { path = "../cli" }`) for `keyring::prompt_password`, `keyring::prompt_store`, `review::prompt` and `ortie::sign_in`; the lead switches it to a git revision at push time. A resolver that does not answer, now an error in io-pim-discovery, is printed under the spinner's failure and leads to the server URL prompt.

Code: src/wizard/search.rs turns the discovered configs into options (protocol, server, sign-in), deduplicated, asking only for the services the build compiles (and running no discovery when it compiles none, an empty set meaning every service to io-pim-discovery), ranked by protocol (People API and Graph contacts, JMAP, CardDAV), server in discovery order, then sign-in (OAuth, token, password with `ortie` on the PATH; password, token, OAuth without). src/wizard/review.rs loops on the review, its Change list naming a server's host, then its URL, only where labels collide. src/wizard/server.rs reads `Server URL:` (https, carddav(s), jmap(s), all rewritten to https; http, no scheme, other schemes, no host and uncompiled protocols refused) then Password or API token. src/wizard/sign_in.rs types the secret through `prompt_password`, tests the account with `check::test_account` under a spinner, stores through `prompt_store` at `cardamum-<name>-carddav` or `-jmap`, runs `ortie::sign_in` as `cardamum-<name>` for OAuth (the broker list after one line when Ortie is missing), and offers Try again, Change the login, Back to the review on a failure. src/wizard/remote.rs ties it together; src/wizard/discover.rs keeps the input prompt, the local flow (still tested once at the end) and the name derivation, and src/wizard/configure.rs settles the account name before discovery so the keys and the Ortie account carry the final, suffixed name. src/wizard/{carddav,jmap,gpeople,msgraph}.rs are gone with their prompts.

Spec: wizard. Added "Auto picks the best-ranked option", "The review names the pick in plain words", "Nothing discovered asks for a server URL", "The user sees where a password goes", "A password is typed once, tested, then stored", "A refusal offers a way on", "OAuth hands off to Ortie". Modified "Input orients the flow", "Discovery is time-bounded", "Connection tested before saving". Removed "One entry per service, then auth", "OAuth folds into the API token", "Stop when nothing is discovered". README, config.sample.toml and the src/main.rs header describe the new run.

Deviations from the brief and the flow:

- A `Login:` prompt remains when no address is known (a bare domain or a server URL as input, discovery advertising no address): the server cannot be signed in without one.
- Every option found is listed under Change, a provider's CardDAV included, where the old wizard collapsed a provider account to its contacts API; Auto still takes the API.
- `Nothing could be discovered for <domain>` prints without the flow's trailing period, the Pimalaya rule for printed messages.
- An empty `Server URL:` reached from the review's `Custom server URL` returns to the review; only the prompt reached because nothing was discovered stops the wizard.
- The CardDAV "None" scheme is no longer offered; a server advertising nothing offers a password and an API token.
- The `rfc9728` feature is not enabled: its metadata fetches run after the discovery deadline, unbounded, and Ortie discovers its own grant.

Verification: `cargo fmt`; `cargo clippy --all-features --all-targets` clean; `cargo test --all-features` 114 passed, 13 of them new in the wizard (dedupe, a server advertising nothing, provider API sign-in, ranking with and without Ortie, provider and server order, login rule, review sentences, Change labels and list, server URL schemes, the http refusal, the remote input parse), the unknown-scheme test moved to src/wizard/remote.rs and the three `AuthCaps` tests gone with the type; `cargo check --no-default-features` and `--features wizard` alone build with the warnings they had in src/account and src/shared, none in the wizard; each of `carddav`, `jmap`, `msgraph`, `gpeople`, `vdir`, `pimdir` with `rustls-ring`, alone and with `wizard`, builds without a warning. The interactive run was not exercised against a server, and the Ortie hand-off is coded against pimalaya-cli's contract while Ortie's side is built in parallel.
