---
cairn: log
change: family-housekeeping
date: 2026-11-15
---

# Family housekeeping

Lines X2 and X6 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15). Docs and build only, no proposal folder. Capabilities moved: none.

- **cairn/verify.sh**: calendula's, copied unchanged; CONTRIBUTING.md names it. Its first run reported 13 C3 violations, the provider reports under cairn/spec/testing/ carrying no `cairn:` frontmatter: each takes the block its README.md and Himalaya's reports carry (`cairn: spec`, `capability: testing`, `status: current`), their prose untouched. It now reports the cairn conformant.
- **pimalaya-config 0.5.2**: required by both dependency tables of Cargo.toml, the Cargo.lock diff that crate alone; an unknown key's error no longer lists a removed key.
- **Already in place**: MIGRATION.md reads "From v0.3 to v0.4"; cairn/changes/ holds the two active proposals alone (card-search, carddav-query-filter), every landed change archived; tests/ holds exit.rs, first_run.rs and account.rs; .github/workflows/tests.yml has the `features` job; the failure code table sits in cairn/spec/commands.md.
