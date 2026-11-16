[![Crates.io](https://img.shields.io/crates/v/cargo-cgp-error-processing.svg)](https://crates.io/crates/cargo-cgp-error-processing)
[![Docs.rs](https://docs.rs/cargo-cgp-error-processing/badge.svg)](https://docs.rs/cargo-cgp-error-processing)

# cargo-cgp-error-processing

**The wording behind the messages of [`cargo-cgp`](https://crates.io/crates/cargo-cgp), CGP's error
toolchain.**

Context-Generic Programming (CGP) extends Rust with pluggable trait implementations, and a mistake
in wiring them reaches you as an error about the code its macros generate. `cargo-cgp-driver`
gathers what the compiler knows about such an error, and this crate turns it into the message the
reader sees: the `[CGP-Exxx]` codes, the root-cause wording, and the dependency chain drawn as a
tree. When a `Rectangle` context lacks the `height` field its area provider reads, this crate writes
the note:

```text
   = note: root cause: [CGP-E110] missing field `height` on `Rectangle`
           this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanCalculateArea` for context `Rectangle`
             └─ [CGP-E102] provider trait impl `AreaCalculator` with context `Rectangle` for provider `RectangleArea`
               └─ [CGP-E110] missing field `height` on `Rectangle`
```

`cargo cgp check` leads with the root cause for the classes it recognizes, and the tool does not yet
reshape every class.

The crate links no compiler internals, so it builds and tests on any toolchain, while the driver
needs the pinned nightly. It is published so that `cargo-cgp-driver` and `cargo-cgp-expand` can
depend on it, and its API serves those two crates alone, so it may change in any release. To read
CGP's errors this way, install `cargo-cgp` instead.

The [CGP website](https://contextgeneric.dev/) explains how to
[read `cargo-cgp`'s output](https://contextgeneric.dev/docs/tooling/cargo-cgp/reading-output) and
what each [error code](https://contextgeneric.dev/docs/tooling/cargo-cgp/error-codes) means, and
teaches CGP itself.
