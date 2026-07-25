# AGENTS.md — the cargo-cgp implementation documentation

This directory documents the internals of `cargo-cgp`. Read [README.md](README.md) for the catalog,
and the knowledge-base-wide [../AGENTS.md](../AGENTS.md) for the rules that govern every category —
the dual-reader prose requirement, the synchronization rule, document-the-present, and the
read-only external references. This file adds the rules specific to the implementation tree.

## What these documents are for

An implementation document is the working note an agent needs to review, debug, or extend one
subsystem of the tool. It explains how the subsystem *works* — the executables and modules involved,
the control and data flow between them, and the contract each side relies on — and, crucially, *why
it is built that way*. Design rationale carries more weight here than in most codebases, because the
non-obvious parts of `cargo-cgp` exist to satisfy cargo's wrapper protocol and the compiler's
`rustc_driver` API, and a reader who does not know the reason will "simplify" a load-bearing detail.

A comparison with related tools belongs in a document whenever one exists, because for most of this
tool's subsystems somebody has already solved the same integration problem and the fastest way to
understand a design decision is to see who else made it. **Clippy** is that tool for most subsystems,
being the reference implementation of this exact compiler integration; `cargo-expand` is the reference
for [the expand command](expand-command.md#comparison-with-cargo-expand), which prints a crate rather
than lints it. Where such a comparison applies, state which tool the design follows, where it
deliberately diverges, and the reason for each divergence. When a divergence is a simplification
`cargo-cgp` has not yet needed to undo — an argument form it does not handle, a compiler hook it does
not install — record it as a gap rather than implying parity, so a later agent inherits the map of what
is missing instead of rediscovering it.

The comparison is not mandatory, though, and a forced one is worse than none. A subsystem with no
counterpart anywhere — a transform particular to reshaping CGP errors, say — omits the section rather
than padding it with a paragraph explaining that no comparison exists.

## The synchronization rule applies here

Keeping a document in sync with the code is part of the change, per [../AGENTS.md](../AGENTS.md). The
source in [`crates/cargo-cgp`](../../crates/cargo-cgp) and
[`crates/cargo-cgp-driver`](../../crates/cargo-cgp-driver) is the single source of truth, above any
document. When you change the structure of the executables, the argument handling, the environment
contract between front-end and driver, or the way the driver accesses the compiler, revise the
matching implementation document in the same change; when you add or remove a test that pins a
behavior a document describes, update that document's Tests section. Verify every claim against the
source — and, for a claim about the behavior of the compiler or of another tool, against the read-only
sources under [`../../../external`](../../../external) — before writing it.

## Document structure

An implementation document follows a predictable shape so an agent can navigate any of them by
habit. It opens with a level-one heading naming the subsystem and a one-sentence summary. The middle
sections describe the subsystem — for a subsystem that spans both executables, a natural order is
one section per executable and then a section per cross-cutting concern (the environment contract,
the compiler-API access). Every document then closes with two standing sections, optionally preceded
by a comparison with related tools:

- **Comparison with related tools** — where the design follows the tool that already solves the same
  problem and where it diverges, with the reason for each divergence, and the gaps where `cargo-cgp` is
  deliberately simpler today. The heading names the tool rather than using this generic wording:
  *Comparison with Clippy* for the compiler-integration subsystems, whose reference is
  `clippy-driver`/`cargo-clippy`, and
  [Comparison with cargo-expand](expand-command.md#comparison-with-cargo-expand) for the expand
  command. Include the section only where such a tool exists; omit it, rather than writing a paragraph
  about its absence, for a subsystem with no counterpart.
- **Tests** — a bullet per test that pins a behavior the document describes, each a link to the test
  with a one-line note on what it verifies. Because the tool's coverage is small, this section is
  also where a reader sees, by omission, what is *not* guarded.
- **Source** — a bullet per source file or module the document covers, each a link, so a reader can
  jump from the prose to the code.

A document may also carry an optional **Further reading** section (before Tests) linking the
authoritative external sources that explain a mechanism it relies on — the Cargo book, the
rustc-dev-guide, a tracking issue. Prefer pointing to such a source over re-explaining a general
mechanism in full, and frame each link with a sentence on what it explains and how it maps to this
tool. Verify a cited URL resolves before committing it.

The **Tests** and **Source** sections are always bullet lists, never flowing paragraphs. A short
lead-in sentence before the bullets is fine; the items themselves are bullets so a reader can scan
coverage and code locations at a glance.

## Level of detail and code snippets

An implementation document gives the high-level picture; the code holds the details, so do not
rehash it line by line. Explain what a subsystem does and why, name the flow and the contracts, and
leave the mechanics of each step to the source. Name an internal function only when a reader needs
it as an entry point into the code. Use a short code snippet — a command line, an argument vector
before and after normalization, the environment a process is launched with — where it makes a
specific behavior concrete, and keep it to the fragment that illustrates the point.

## Known gaps and limitations

Record a limitation or a deliberate simplification where the relevant document's own structure calls
for it — most naturally in the comparison section where a document has one, since today most gaps are
behaviors the reference tool handles and `cargo-cgp` does not yet, and otherwise beside the behavior
they qualify. Describe the behavior as it currently is, say what the
fuller behavior would be, and remove the note in the same change that closes the gap, per the
synchronization rule. Do not leave a fixed limitation described as if it still holds.
