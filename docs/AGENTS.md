# AGENTS.md — the cargo-cgp knowledge base

This file governs how to write and maintain the documents under `docs/`. Read
[README.md](README.md) first for what the knowledge base is and how it is organized, and the
repository-root [AGENTS.md](../AGENTS.md) for the code itself. The rules here apply to every
category; a category may add its own rules in its own `AGENTS.md` (the implementation category does,
in [implementation/AGENTS.md](implementation/AGENTS.md)).

Write in the dual-reader prose style — **load the `/dual-reader-prose` skill** before writing or
revising any document here, and follow it: open every paragraph with a self-contained topic
sentence, and frame any list with a sentence before and, where useful, after. A document here is
read both by an agent scanning for one fact and by an agent reading a subsystem end to end, and the
style serves both.

## The synchronization rule

A document must stay in sync with the code, and keeping it in sync is part of the change, not a
follow-up. **The source is the single source of truth**, above any document. When you change how the
tool behaves — its argument handling, the environment variables the executables agree on, the way
the driver drives the compiler, the crate or module structure — revise the matching document in the
same change. A document that describes a design the code no longer has is worse than no document,
because the next agent will trust it. Verify every claim against the source before you write it,
rather than transcribing another document or working from memory.

The rule extends to the code's own inline documentation. Reading a module closely enough to document
it is exactly when to fix its inline docs, so in the same pass add a one-line `///` to any public
item that lacks one, correct a comment that no longer matches the code, and delete a comment that
only restates the obvious. Keep inline docs terse and leave the deeper reasoning to the knowledge
base; a one-line doc comment that links out to a document beats a paragraph inlined in the source.

## Document the present, not the history

Describe how the code works now. Record current limitations plainly where the relevant document
calls for it, but do not narrate how the design used to be: delete superseded wording outright
rather than leaving "previously", "renamed from", or "used to" traces. An agent reading a document
should learn the current state, not archaeology. Git history is where the evolution lives.

## The external references are read-only ground truth

`cargo-cgp` is modeled on Clippy and built against the compiler's internal API, and both are
available as local sources: the Rust compiler at [`../external/rust`](../../external/rust) and Clippy
at [`../external/rust-clippy`](../../external/rust-clippy). When a document makes a claim about how
`rustc_driver` behaves or how Clippy does something, verify it against those sources rather than from
memory, since the compiler's internals shift between nightlies. Treat them as read-only: cite them,
do not edit them, and do not create a dependency on them. The same holds for the parent `cgp`
repository at [`../cgp`](../../cgp), which this project reads but never modifies.

## Show the example behind an error message

When a document mentions an error message tied to a specific example, include that example's code in
the document and explain both what it does and what root cause produces the error. A reader who meets
a rewritten `[CGP-Exxx]` headline or a raw compiler error should be able to see, in the same
document, the small program that triggers it — the `delegate_components!` block, the provider impl,
the `check_components!` entry — rather than reconstruct it from the message alone. Show the snippet,
say what it is wiring or declaring, and then name the mistake the message is really about: the field
the context never derives, the component it omits, the redirect that resolves to nothing. The message
and its cause belong together, because the point of this knowledge base is to explain *why* an error
reads the way it does, and an error quoted with no example behind it cannot be checked or understood.

## Keep backticks well formed

Malformed backticks are the most common way a document here renders wrong, so treat them carefully
and re-check them after every edit. Keep the opening and closing backticks of an inline code span on
the **same line** — never let a line break fall between them, which breaks the span. When a sentence
with inline code would wrap, wrap it elsewhere or let the line run long; do not split the code span.
For a fenced code block, put the triple-backtick fences on their own lines with a blank line before
the opening fence and after the closing one, so the block is recognized rather than folded into the
surrounding paragraph. **After editing any markdown document, read the text back and confirm every
backtick is well formed** — each inline span opened and closed on one line, and each fenced block
delimited by matched triple-backtick lines with the blank lines around them.

## Registering a document

Every document registers itself in its category's `README.md` catalog in the same change that
creates it, so the catalog is never behind the tree. When you add a whole category, create its
directory with a `README.md`, give it an `AGENTS.md` if it needs rules of its own, and register the
category in the knowledge-base [README.md](README.md).
