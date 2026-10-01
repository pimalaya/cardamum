---
cairn: log
change: gpeople-rename
landed: 2026-10-01
---

# The Google People backend is `gpeople`

The People backend follows the vendor-in-the-name rule that `gmail` (Himalaya) and `gcal` (Calendula) already follow, since "people" alone names no provider. The cargo feature, the account config table and the command all moved from `people` to `gpeople`, with no alias: a 0.2 config naming `people` must be edited. The module moved from `src/people/` to `src/gpeople/` and every `People*` type became `Gpeople*`, cardamum's own and io-gpeople's alike.

The dependency moved from io-people 0.3.0 to io-gpeople 0.4.0, which renamed its types to `Gpeople*` the same way. io-jmap and io-msgraph moved to 0.4 with no code change.

Capabilities moved: **backends** and **config** (the feature and table name), **projection** (the link to the moved module). The manual test reports under `spec/testing/` keep the `people` commands they were run with.
