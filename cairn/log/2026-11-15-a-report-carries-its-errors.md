---
cairn: log
change: a-report-carries-its-errors
date: 2026-11-15
---

# A report carries its errors

`carddav report query` and `carddav report multiget` name each card the server answered without its data in an `errors` list and exit 3, closing what a-result-carries-its-errors left open and live finding S4. Capabilities **commands** and **backends** moved.

- **src/carddav/report/entries.rs**: `CarddavCardEntriesOutput` carries `errors` (`cardamum-carddav-report-query` and `-multiget` schemas gain it, the item error typed by the code vocabulary); its text prints the `Error:` lines last through `pimalaya_cli::error::ItemErrors`, after the table and the truncation line and one blank line, none leading an output of errors alone.
- **src/carddav/report/query.rs**: each `CarddavCardListOk::unreadable` row is an entry through `carddav::backend::unreadable_row`, the function `card list` uses (made `pub(crate)`), so the codes, messages and sources are the same; `truncated` stays a field and adds no entry, since `--limit` asks for the cut.
- **src/carddav/report/multiget.rs**: each `CarddavCardMultigetOk::unreadable` row is an entry, a 404 or 410 row `card-not-found` ("The server holds no such card, status 404"), the others through `unreadable_row`.
- **Evidence**: `a_query_names_the_rows_it_could_not_read` and `a_multiget_names_the_cards_it_could_not_read` fail with the entries left out (checked by emptying them in the two commands); `a_multiget_reading_every_card_prints_no_error`, `the_errors_print_last_after_one_blank_line`, `errors_alone_print_without_a_leading_blank_line`.

CHANGELOG (**Breaking:** the exit 3), MIGRATION.md. Spec: commands ("A CardDAV report names the cards it could not read" added, "Exit statuses" and "A JSON error carries a stable code" modified), backends ("A server's refusal carries its code" modified); cairn/spec/testing/carddav-specific.md records S4 fixed. Left open: an id a multiget named that the server leaves out of the multistatus altogether is still dropped, as io-webdav returns only the rows it got; matching the ids given against the hrefs answered needs the server's percent-encoding settled first.
