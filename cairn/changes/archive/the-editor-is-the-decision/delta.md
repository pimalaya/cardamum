---
cairn: change
id: the-editor-is-the-decision
status: landed
created: 2026-08-31
---

# Delta

## ADDED Requirements

### Requirement: A build can be composed, and captured
`card build` SHALL take `-i/--interactive` and `--composer <COMMAND>`, the same pair the two write verbs carry, and SHALL print what the composer wrote rather than sending it anywhere. This is where a card is judged before it is written: the menu that used to offer a look is gone, and a build is the look.

It SHALL resolve the account only when `-i` is given without `--composer`, the composer command living in the configuration. A `--composer` line keeps the command account-free, which is what `card build` is for.

`-o/--output <PATH>` SHALL write the card to that file instead of printing it, answering a message rather than the card. The flag earns its place on this command alone: the composer inherits stdout, so `card build -i > card.vcf` hands the editor the file as its terminal, and with `-i` there is no redirection to fall back on.

An abandoned interactive build SHALL print nothing at all, and exit as a success. The output of this command is a vCard someone pipes onwards, and a line saying no card was built is not one.

### Requirement: A source carries a card or it is refused
A source that holds nothing but whitespace SHALL be refused, naming where it was read from, rather than read as a card. Handing a backend an empty body is never what was meant, and an abandoned `card build -i` prints nothing, which pipes straight into the next command.

A source holding several vCards SHALL be refused when a field flag is set. A flag rewrites the card the parser reads first, so the cards behind it would be dropped without a word. With no flag the source passes through as it was written, several cards included, which is the promise every other source keeps.

## MODIFIED Requirements

### Requirement: The composer's own exit is the decision
When the composer exits, what it left in the file SHALL settle the edit, and nothing SHALL be asked after it. A file that came back changed is the card, and the command writes it. A file the composer emptied, and one handed back byte for byte as it was given, are an edit given up on: nothing is written, and nothing failed. A non-zero exit status is a failure and SHALL be reported as one.

Those three outcomes SHALL be the whole protocol, so a composer owning its own save and discard is not second-guessed by a menu it cannot see, and a plain editor keeps the meaning its own quit already has: a graphical editor says no by emptying the file or by exiting non-zero, and `nvim` says it with `:q!`.

Abandoning SHALL be the same on a create and on an update, and SHALL exit as a success: the card is written or it is not, and which verb asked is already on the command line. `card build` SHALL print nothing when abandoned, the two write verbs SHALL say that nothing was written.

### Requirement: A composed card is checked before it is written
What the composer wrote SHALL be checked against its own version's RFC contract, through vcard-rs's validator rather than a look at its first line. Reading a card is liberal and this is the strict half: a 4.0 card missing its required `FN`, a 2.1 one missing its required `N`, a property the version does not define, are all caught here rather than by the server or by nobody.

A card that does not pass SHALL have its violations printed and SHALL offer to re-open the editor, defaulting to yes. This is the one thing a composer cannot be trusted with, a plain editor happily handing back a card missing its `FN`, and it is not a menu: the only question is whether to fix it, and declining is an error rather than an abandon.

A card built from flags with no source SHALL be checked the same way and refused, naming the violations, whether it was `card create` or `card build` that built it. A vCard given on the command line SHALL NOT be, going to the backend as it was written: that is the promise the protocol-specific commands already make, and second-guessing a card someone handed over is not cardamum's to make.

### Requirement: An edit is never lost
When a composed card cannot be written, whether the check was declined or the backend rejected it, the temporary file SHALL be kept and the error SHALL name it: `Cannot edit vCard <path>`, which says what failed and where the work is in one line, the path being the recovery. A person who spent a minute in an editor never loses it to a failure that happened afterwards.

An abandoned edit SHALL drop the file instead: an emptied one holds nothing, and an untouched one holds only what cardamum put there, so neither is anything to lose, and removing them is what keeps the temporary directory clear of every abandoned run.

### Requirement: A card can be built without an account
`card build` SHALL take the same vCard source, the same field flags and the same composer as `card create`, apply them in the same order, and print the resulting vCard rather than sending it anywhere. It SHALL reach no backend.

It SHALL read no configuration, unless `-i` is given without `--composer` and the configured composer is the one thing it needs. A vCard can therefore be formatted on a machine holding no configuration at all, and the account a card subcommand runs against SHALL be resolved by that subcommand rather than ahead of the whole `card` family.

It SHALL check what `card create` would check: a card built from flags with no source is validated and refused, a vCard given as a source passes through untouched. A weaker rule would make `card build ... | card create -` a way past the guard, the piped card arriving at `create` as a source, which `create` does not check.

A build given no source, no field flag and no `-i` has nothing to build and SHALL say so.

## REMOVED Requirements

### Requirement: The composer hands back to a menu, not to a write
Removed: the menu is gone. A composer that owns its own save and discard cannot express either through a prompt cardamum shows after it exits, and a graphical one shows it where nobody is looking. What it did buy, a look at the card before it left, is `card build -i`.
