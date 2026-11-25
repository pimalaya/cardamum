---
cairn: log
change: config-sample-on-himalayas-shape
landed: 2026-11-25
---

# Aligned config.sample.toml on Himalaya's

## What landed

- config.sample.toml: Himalaya's shape, 97 lines down to 55: one commented key per line with a one-line note, no `##` lines, alternatives in brackets (`proxy.url`, `tls.provider`, `tls.rustls.crypto`, `carddav.server`, `jmap.server`, `msgraph.user-id`), `proxy` and `tls` as flat keys, one-line headings, the per-backend `alpn`, `tls` and `proxy` keys once under a Network heading.
- Sections left: CardDAV (Basic, then a Bearer paragraph with `carddav.home`, Google's form in its note), JMAP (Basic, then Bearer), Microsoft Graph, Google People, vdir, pimdir. The Google CardDAV and Fastmail sections went (Fastmail is a README recipe, Google CardDAV a note), CardDAV without credentials became a note on `carddav.auth.basic.username`, and the per-backend `tls` and `alpn` repeats of jmap, msgraph and gpeople went.
- src/config.rs: `the_sample_configuration_parses_with_each_variant` ported from Himalaya (`SampleLine`, `sample_paragraphs`, `sample_ranks`, `sample_variant`), loading every build, the no-backend one included, and still asserting a backend once uncommented; the `sample` feature of the pimalaya-config dev-dependency dropped.
- tests/common.rs: `sample_section` on Himalaya's (one-line headings, first line per field, a secret's later alternative skipped), reading a section to the next heading and taking an alternative auth paragraph when an override names it. The Fastmail suite uses the JMAP section with Fastmail's session URL and the CardDAV section with Fastmail's host; the Google suite the CardDAV section with its Bearer paragraph and Google's home.
- Capabilities moved: config (the sample requirement).
