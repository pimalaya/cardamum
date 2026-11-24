---
cairn: log
change: the-live-night-findings
date: 2026-11-24
---

# The live night findings, D8

One item of cairn/changes/the-live-night-findings found by the second live night (workspace .lanes/live/journal-2.md, the cardamum lines).

- **D8, `gpeople other-contacts copy`.** Its copy mask was the read mask of the other-contacts listings, `metadata` included, and People refused every copy: `HTTP 400 INVALID_ARGUMENT: Request field 'metadata' not allowed for other contacts mutate requests`. The copy now sends `OTHER_CONTACT_COPY_FIELDS`, `names,emailAddresses,phoneNumbers`, the three fields `copyOtherContactToMyContactsGroup` copies; the listings keep their mask. Pinned by `gpeople::other_contact::fields::tests::the_copy_mask_holds_only_names_emails_and_phones`, which failed on `Metadata` before the fix. Live on google: `gpeople other-contacts copy otherContacts/c42929557656026705` created `c492007068989957982`, deleted after.

Gate: `cargo fmt --check` clean; `cargo clippy --locked --all-targets -- -D warnings` clean over the 11 feature sets of .github/workflows/tests.yml; `cargo test --locked --all-features` 386 passed, 19 ignored; the two single-backend CI steps green (151 and 10 passed); live suites google (10) and microsoft (5) passed.

Capabilities moved: backends (an other contact copy sends the copy mask).
