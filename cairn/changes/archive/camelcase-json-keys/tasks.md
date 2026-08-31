---
cairn: tasks
change: camelcase-json-keys
---

# Tasks

- [x] `rename_all = "camelCase"` on every output type with named fields, row and payload types included
- [x] leave the transparent newtypes and the untagged enum alone, neither having a field to rename
- [x] leave the provider passthrough spellings alone, and verify in the emitted schema that the container attribute does not reach them
- [x] update every doc comment enumerating a key, the doc comment being the help page
- [x] regenerate the 57 schemas and read the property names back
- [x] CHANGELOG, including the `card update` entry that already spelled `kept_properties`
- [x] cargo fmt, check, test and clippy on the full feature matrix, plus the per-backend sweep
