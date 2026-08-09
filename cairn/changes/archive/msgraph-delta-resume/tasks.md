---
cairn: tasks
change: msgraph-delta-resume
---

# Tasks

- [x] io-msgraph: `MsgraphContactsDelta::from_link` plus the `contacts_delta_link` client method
- [x] cardamum: `contact delta --delta-link <URL>`, conflicting with `--folder` / `--select`
- [x] cardamum: `request` prints nothing in text mode for an empty response body
- [x] cardamum: the shared `-k` resolver rejects an empty id
- [x] cargo fmt, clippy, feature matrix, tests
- [x] Re-test live against `msgraph`: a full delta round (initial, change, resume), the guards, the raw verbs
- [x] Update the reports, fold the delta, log and archive
