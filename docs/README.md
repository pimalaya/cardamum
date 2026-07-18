# docs/

Development memory of the cardamum CLI: architecture notes that outgrow the `main.rs` header, plus plans and their outcomes. The narrative architecture (where cardamum fits, the backends, the command families, backend selection, vCard projection, the configuration and wizard model, output) lives in the [`main.rs`](../src/main.rs) crate header; this folder holds the reference detail and the development log.

- [architecture.md](architecture.md): the module layout map, command conventions, and the deeper mechanics of the wizard's email discovery and the CardDAV discovery routes.
- [contacts-mapping.md](contacts-mapping.md): the policy for projecting the API backends (Microsoft Graph, Google People) to and from the vCard document of record — which fields are managed, minted or left alone.
- [custom-data.md](custom-data.md): how vCard properties with no provider slot survive a round-trip, stashed verbatim in a Graph extended property / People clientData entry.
- [merged-view.md](merged-view.md): why group memberships are structural addressbook data rather than a card property.
- [specific-apis-design.md](specific-apis-design.md): design of the per-backend protocol-specific subcommands (Himalaya-style, matching each remote protocol's structure) — flat WebDAV methods for CardDAV, resource-nested for msgraph/people/jmap, item subcommand for vdir; the iteration plan for implementing them.
- [testing/provider-test-plan.md](testing/provider-test-plan.md): followable checklist to deeply exercise every shared command against a real provider, one report per `(backend, provider)`.
- [testing/carddav-fastmail.md](testing/carddav-fastmail.md): CardDAV on Fastmail — first completed shared-command test report.
- [testing/carddav-icloud.md](testing/carddav-icloud.md): CardDAV on iCloud — card CRUD validated (the id fix confirmed on a second `.vcf` server); iCloud vCard-input strictness noted.
- [testing/carddav-google.md](testing/carddav-google.md): CardDAV on Google over OAuth Bearer — id fix confirmed on a third server; Google resource-id reassignment (delete-by-create-id gap) recorded.
- [testing/msgraph-microsoft.md](testing/msgraph-microsoft.md): Microsoft Graph over OAuth Bearer — full addressbook + card CRUD working; no bugs; Graph-assigned UID, no folder metadata / If-Match noted.
- [testing/google-people.md](testing/google-people.md): Google People API over OAuth Bearer (`contacts` scope) — full contact-group + card CRUD working; no bugs; groups-as-books, `myContacts` ubiquity, etag-guarded delta updates noted.
- [testing/jmap-fastmail.md](testing/jmap-fastmail.md): JMAP on Fastmail over OAuth Bearer — card CRUD working after porting the JSContact projection from calcard to vcard-rs (calcard's non-standard `vCard` container was rejected by Fastmail); AddressBook mutations forbidden server-side.
- [testing/vdir-local.md](testing/vdir-local.md): vdir local filesystem — shared API **and** `vdir` subcommands both working against a throwaway `/tmp` instance; no bugs; byte-faithful storage, clear-metadata no-op (V1), `--if-match` ignored (V2).
