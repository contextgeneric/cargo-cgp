# Pending issues

This directory tracks the work `cargo-cgp` has not yet done — the problems the tool is meant to
solve but does not solve today. Unlike the rest of the knowledge base, which describes how the tool
*currently* works, this category deliberately describes behavior that is absent or intended: it is a
checklist of gaps, not a record of the design. Each entry names a gap, points at the evidence for
it, and says what closing it requires, so the next agent can pick up a concrete piece of work
without rediscovering it.

Every issue is backed by a test case. An issue is worth reporting only if a fixture under
[`tests/ui/`](../../tests/ui) reproduces it, so each entry links the fixture that exposes it, and a
class of issue with no reproducing fixture is treated as **resolved** — deleted from these
documents rather than kept as a hypothetical. This keeps the checklist honest: it lists problems a
reader can see happen, not problems we imagine. When an issue is fixed, delete its entry and move its
fixture to the passing category (below); the git history records the evolution, and a stale "fixed"
note is worse than none.

Two things are out of scope. First, the classes the tool **already** handles well: `cargo cgp check`
compiles the workspace through a `rustc_driver` wrapper that both injects `-Znext-solver=globally` —
surfacing the CGP dependency bounds the default solver hides — and rewrites the CGP error classes it
recognizes into a root-cause-first form stamped with `[CGP-Exxx]` codes. Those resolved classes are
recorded as `acceptable/` fixtures (and in the CGP error catalog), not as issues here; this directory
tracks only what the tool does *not* yet do well — a cause it cannot recover, or one it recovers but
presents poorly. Second, this catalog is only about the diagnostics CGP produces; a plain Rust or
Cargo error that has nothing to do with CGP is not `cargo-cgp`'s problem to reformat and is not
recorded here.

## Organization

The issues split along one axis: whether the root cause is *recoverable* from the tool's output at
all, or merely hard to read. That axis matters because it decides who the issue is for and how much
the tool must do to close it — recovering absent information needs the compiler-internal foothold,
while re-presenting present information is post-processing. The `tests/ui/` fixtures are grouped into
the same categories, so a fixture's directory names the kind of problem it exposes.

- [Hidden root cause](hidden-root-cause.md) — **tool-oriented**, about *sufficiency* rather than
  format. It catalogs the edge cases where `cargo-cgp` does not emit enough information for any
  downstream consumer — a formatter, an IDE, or an AI agent — to identify the root cause from the
  output alone, no matter how that output is processed. These are the cases that justify the tool's
  compiler-internal access, because only a tool reading the compiler's own state can supply what the
  ordinary text output has lost. It currently has **no reproduced case** — both known archetypes are
  defeated by flags the driver injects (`-Znext-solver=globally` and `--verbose`), so the
  `tests/ui/hidden-root-cause/` directory is empty and absent until a genuinely unrecoverable case is
  found; the document records the two defeated archetypes so they are recognized again.
- [Usability issues](usability.md) — **human-oriented**, about *readability*. It lists output that
  does carry the root cause but buries it — foremost the sheer verbosity of a CGP compile error — so
  a reader must wade through volume, encoding, and generated-type noise to reach a cause that is
  nonetheless present. Its fixtures live in [`tests/ui/usability/`](../../tests/ui/usability).

The dividing line is a single test: if no amount of downstream processing of the text could
reconstruct the root cause, the issue is a hidden root cause; if a sufficiently careful reader or
tool could, it is a usability issue. Both draw their evidence from the fixtures and read them against
the upstream [CGP error catalog](https://github.com/contextgeneric/cgp/blob/main/docs/errors/README.md), which maps every error class
CGP produces and, class by class, whether the root cause is present in the output or suppressed.

Two further categories hold the fixtures that are *not* open problems, and neither has a matching
issues document because there is nothing to fix. [`tests/ui/ok/`](../../tests/ui/ok) is the
clean-compile baseline — correctly-wired programs that check with empty output.
[`tests/ui/acceptable/`](../../tests/ui/acceptable) is where an *error* fixture graduates once
`cargo-cgp` presents its cause well enough: the typed root-cause resolver has already carried the
whole check-trait-failure family there, so `acceptable/` holds the reformatted errors that clear the
usability bar, grouped into concept sub-directories. Their snapshots are the standing proof the tool
keeps producing good output. As a usability issue is closed, its fixture graduates from `usability/`
into `acceptable/` — a plain move of its `.rs`/`.cgp.stderr`/`.rust.stderr`/`.expand.rs` set, since the
snapshots are independent of the fixture's directory.
