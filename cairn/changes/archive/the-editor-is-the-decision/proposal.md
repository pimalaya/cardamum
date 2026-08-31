---
cairn: change
id: the-editor-is-the-decision
status: landed
created: 2026-08-31
---

# The editor is the decision, and a card can be judged before it is sent

## Why

[card-composer](../archive/card-composer/proposal.md) put a menu after the editor: `Save`, `Preview`, `Edit again`, `Abort`. It made sense while the only composer was a terminal editor cardamum spawned itself, and it stops making sense the moment the composer is a program with a mind of its own. A graphical editor has its own save and its own discard. Asking again, in the terminal, after its window closed, second-guesses a decision the person already took, in a place they are no longer looking.

The menu is also a decision cardamum cannot take on anyone's behalf. It stands between the editor and the write, so the composer cannot express `Save` except by cardamum asking, and it cannot express `Abort` at all: today aborting is a menu entry, while a composer exiting non-zero is reported as a failure and an emptied file as an abandon. Three ways of saying no, two of them undocumented, none of them available to the composer that actually knows.

What replaces it already exists in the protocol and in every editor anyone uses: quitting without writing means no. `git commit` has worked that way for twenty years, and its empty message is exactly the emptied file cardamum already reads as an abandon.

What the menu did buy was a look at the card before it left. That is worth keeping, and it belongs to `card build`, which was made for exactly that and is one flag short of it.

## What

**The composer's own exit is the decision.** A file that came back changed is the card and is written. A file the composer emptied, or handed back byte for byte as it was given, is an edit given up on: nothing is written, and nothing failed. A non-zero exit is a failure and says so. Three outcomes, one rule, the same for tCard, for `nvim` and for the graphical editor that does not exist yet, which now has a way to say no: empty the file, or exit non-zero.

**`card build -i`** opens the composer on the card the source and the flags built, and prints the result rather than sending it. This is the preview the menu used to offer, with the advantage that what comes out of it is a file, a pipe or a look, rather than a decision taken in a prompt. `card build -i -o card.vcf`, read it, then `card create -k <AB> card.vcf`.

**`card build -o <PATH>`** writes the card to a file instead of stdout. It earns a flag here where it would not anywhere else: the composer inherits stdout, so `card build -i > card.vcf` hands the editor the file as its terminal. With `-i` there is no redirection available, and the command needs a way out of its own.

**`card build -i` reads the configuration, and only then.** The composer command lives there. `--composer <COMMAND>` names one outright and keeps the command account-free, which is what it was for; without it, `-i` resolves the account for that one value.

**Two silent losses, closed on the way.** A source holding several vCards, with a field flag set, kept the first card and dropped the rest, because a flag rewrites the card the parser reads first: that is now refused. A source holding nothing but whitespace was read as a card and handed to the backend as an empty body: that is now refused too, and it stops being theoretical the moment an abandoned `card build -i` prints nothing into a pipe.

## What this costs

The card that leaves an interactive create or update is now whatever the editor saved, with no last look. That is the trade the menu was paying for, and `card build -i` is where the look moved: two commands instead of one prompt, and the card is a file in between rather than a decision in a prompt.

Cardamum still checks what it is about to write, whichever composer wrote it, and still offers to re-open on a card that does not pass. That is not a menu and does not become one: it is the one thing a composer cannot be trusted with, since `nvim` will happily hand back a card missing its `FN` and something has to refuse it.

## What this is not

The protocol is unchanged: a command, a path, streams inherited, edits in place. Nothing here teaches cardamum about an editor, and the graphical editor is still a command it spawns.

`card build` still writes to no backend, and `card create` and `card update` still write to nothing else. No verb changed what it promises.
