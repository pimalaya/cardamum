# docs/

Development memory of the cardamum CLI: architecture notes that outgrow the `main.rs` header, plus plans and their outcomes. The narrative architecture (where cardamum fits, the backends, the command families, backend selection, vCard projection, the configuration and wizard model, output) lives in the [`main.rs`](../src/main.rs) crate header; this folder holds the reference detail and the development log.

- [architecture.md](architecture.md): the module layout map, command conventions, and the deeper mechanics of the wizard's email discovery and the CardDAV discovery routes.
- [contacts-mapping.md](contacts-mapping.md): the policy for projecting the API backends (Microsoft Graph, Google People) to and from the vCard document of record — which fields are managed, minted or left alone.
- [custom-data.md](custom-data.md): how vCard properties with no provider slot survive a round-trip, stashed verbatim in a Graph extended property / People clientData entry.
- [merged-view.md](merged-view.md): why group memberships are structural addressbook data rather than a card property.
