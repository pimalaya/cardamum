---
cairn: log
change: a-text-output-ends-with-a-newline
date: 2026-11-15
---

# A text output ends with a newline

`card read` ends its text with a newline when the card's last line lacks one, and the `jmap`, `msgraph` and `gpeople` `request` text ends with one. Task C28 of cli-alignment-2026-11-15, live findings F2 and J3. Capability **commands** moved.

- **src/shared/card/read.rs**: the newline added when the card does not end with one. **src/shared/raw_json.rs**: `writeln!`.
- **Kept raw**: `carddav get`, `vdir item get` and `card build` print the bytes asked for.
- **Evidence**: `the_text_view_ends_with_a_newline` and `an_empty_response_prints_nothing` fail on the code before.

CHANGELOG (Fixed). Spec: commands ("Text output is plain text").
