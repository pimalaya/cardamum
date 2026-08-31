---
cairn: log
change: the-editor-is-the-decision
landed: 2026-08-31
---

# The editor is the decision, and a card can be judged before it is sent

The composer landed this morning with a menu after the editor: `Save`, `Preview`, `Edit again`, `Abort`. It was right for the only composer that existed then, a terminal editor cardamum spawned itself, and wrong for the next one: a graphical editor has its own save and its own discard, and a terminal prompt after its window closed second-guesses a decision already taken, somewhere nobody is looking. The menu is gone, never having been released.

**Three outcomes, one rule** (shared/card/composer.rs): a file that came back changed is the card, a file the composer emptied or handed back byte for byte is an edit given up on, and a non-zero exit is a failure. Nothing is asked after the editor. The same protocol serves tCard, `nvim` (`:q!` leaves the seed untouched, which is a no) and the graphical editor that does not exist yet, which now has two ways to say no rather than none. `git commit` has worked this way for twenty years.

**What the menu bought moved to `card build -i`** (shared/card/build.rs): the composer opens on the card the source and the flags built, and what comes back is printed rather than sent. `card build -i -o card.vcf`, read it, then `card create -k <AB> card.vcf`. The look is now a file, a pipe or a terminal read, instead of a decision taken inside a prompt.

**`-o/--output` earns a flag on this one command**: the composer inherits stdout, so `card build -i > card.vcf` hands the editor the file as its terminal. With `-i` there is no redirection to fall back on, which is exactly why the same flag was refused on `card create` two changes ago, where stdout is free and the product is an id.

**The configuration is read only when it is the missing piece**: `-i` with `--composer` stays account-free, `-i` alone resolves the account for `card.composer` and nothing else. Bare `card build` still runs on a machine that has never seen a configuration, verified with an empty `HOME`.

**Two silent losses closed on the way** (shared/card/vcard.rs, shared/card/fields.rs). A source holding nothing but whitespace was read as a card and handed to the backend as an empty body, exit 0: `printf '' | card create -k <AB> -` was a clean success that wrote nothing anyone meant. And a source holding several vCards, with a field flag set, kept the first card and dropped the rest without a word, because a flag rewrites the card the parser reads first. Both are refused now, and the first stopped being theoretical the moment an abandoned `card build -i` prints nothing into a pipe.

**What is not a menu, and stays**: cardamum still checks what it is about to write, whichever composer wrote it, and still offers to re-open on a card that does not pass. A plain editor happily hands back a card missing its `FN`, and something has to refuse it. The only question asked is whether to fix it, and declining is an error rather than an abandon.

Capabilities moved: commands (two requirements added, four modified, one removed), backends (one sentence, the menu no longer sitting between the editor and the connection).

Verified: the four composer outcomes end to end with `--composer true`, `--composer false`, `--composer "truncate -s 0"` and a `sed -i` that edits, an abandoned build printing nothing at exit 0, `-o` writing the file and answering a message, `--json` still refusing to spawn, the temporary file removed on every abandon, both refusals bailing, and a two-card source still passing through untouched with no flag. 72 tests green on the full feature set, clippy clean.
