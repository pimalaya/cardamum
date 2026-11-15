---
cairn: delta
change: a-jmap-version-is-a-stable-digest
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

### Requirement: A JMAP card's version is a stable digest
(backends) The JMAP backend SHALL report as a card's `etag`, in `card list` and `card read`, the BLAKE3 digest of the ContactCard's JSON in lowercase hex, its keys in serde_json's sorted order, JMAP having no per-card version. The function SHALL be a fixed one, never Rust's `DefaultHasher`, whose algorithm may change from one Rust release to the next: a version one build prints is passed back as `--if-match` to another, and compared with the digest of the card read before the write ("An update guards on the version it read" in commands).

## MODIFIED Requirements

### Requirement: An update guards on the version it read
(commands) JMAP compares with the BLAKE3 digest of the JSON of the ContactCard the update reads as its patch base, where it read "the hash of the ContactCard"; the code table's `precondition-failed` row reads "the Graph `changeKey` or JMAP digest". The rest is unchanged.

## REMOVED Requirements

None.
