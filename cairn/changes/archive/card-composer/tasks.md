---
cairn: tasks
change: card-composer
---

# Tasks

- [x] Promote vcard-rs and getrandom to unconditional dependencies, dropping them from the backend feature lists.
- [x] Extract the UUIDv4 generator out of the CardDAV backend into shared/uuid.rs.
- [x] Add `composer` to `CardConfig`, merge it onto the runtime `Account` and resolve it with a getter that names both ways of setting one.
- [x] Write shared/card/composer.rs: the temporary file, the spawn with every stream inherited, the read back, and the draft the caller finishes.
- [x] Write shared/card/fields.rs: the field flags and how they are written onto a card, replacing each property they name and merging `N`.
- [x] Turn shared/card/vcard.rs into the source module: `read_source`, `blank_card` and the version argument.
- [x] Rebuild `card create` and `card update` on the one source-flags-composer pipeline, both positionals optional.
- [x] Answer an untagged output on both verbs, covering the write and the abandoned edit, and keep the registry in step.
- [x] Document the option in config.sample.toml and the change in CHANGELOG.md.
- [x] Check and clippy every backend on its own, and the full feature set.
- [x] Fold the delta into the spec and write the log entry.
