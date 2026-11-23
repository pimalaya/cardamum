---
cairn: log
change: the-live-night-fixes
date: 2026-11-23
---

# The live night fixes

Three bugs the live test night of 2026-11-23 met on real accounts, fixed without a proposal (each a clear bug, the behaviour the spec now states). Evidence: .lanes/live/journal.md, the cardamum lines.

- **`jmap contact-card get` and `jmap address-book get`**: `ContactCard/get` of `DCIo nonexist` on Fastmail printed `{"list":[…],"state":"13677"}` and exited 0, the server's `notFound` dropped. The raw get (src/jmap/raw.rs) now keeps `notFound`, and each id in it is an `errors` entry coded `card-not-found` or `addressbook-not-found`, `notFound` in `sources`, exit 3, as Himalaya's JMAP gets and cardamum's `carddav report multiget` report theirs. Pinned by `jmap::cli::tests::an_id_a_get_does_not_find_is_an_item_error`.
- **`gpeople connections update`**: the command fetched the person and sent its current etag whatever the JSON named, so `{"etag":"bogus",…}` was applied over a newer person. The JSON's etag is now the guard, People refusing a stale one (400 `FAILED_PRECONDITION`, exit 3); the current etag is fetched only when the JSON names none. Pinned by `gpeople::cli::tests::an_update_sends_the_etag_the_caller_gave`.
- **`gpeople request`**: `post people:createContact` was refused `request-origin`, the url crate reading `people:` as a scheme of a null origin. An argument holding `://` is a full URL, any other joins onto the People base as `./<path>`; the off-origin refusals keep their tests. Pinned by `gpeople::request::tests::a_custom_method_path_resolves_on_the_people_origin`.

Each pinning test failed before its fix. Gate: `cargo fmt --check` clean; `cargo clippy --locked --all-targets -- -D warnings` clean over the 11 feature sets of .github/workflows/tests.yml; `cargo test --locked --all-features` 378 passed, 19 ignored; the two single-backend CI steps green (147 and 10 passed).

Capabilities moved: commands (raw-faithful specific commands, the raw request origin rule).
