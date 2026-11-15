---
cairn: change
id: the-people-resources-are-plural
status: landed
created: 2026-11-15
---

# The People resources are plural

## Why

Himalaya's `gmail` names its resources by the API's plural nouns (`messages`, `labels`, `threads`), the singular a hidden alias; cardamum's `gpeople` used the singular (`connection`, `contact-group`, `other-contact`), though the People API itself names them `people.connections`, `contactGroups` and `otherContacts`. Finding F23, task C7 of cli-alignment-2026-11-15.

## What

- `gpeople connections`, `gpeople contact-groups`, `gpeople other-contacts`; the singular names hidden aliases, the visible `people`, `contacts`, `group`, `groups` and `other` kept. `gpeople profile` reads one user and stays singular, as Himalaya's `gmail profile`.
- The `json-schema` keys follow the command path (`cardamum-gpeople-connections-list`), where v0.3 wrote the singular: MIGRATION.md says it.
