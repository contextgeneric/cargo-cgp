[![Crates.io](https://img.shields.io/crates/v/cargo-cgp-expand.svg)](https://crates.io/crates/cargo-cgp-expand)
[![Docs.rs](https://docs.rs/cargo-cgp-expand/badge.svg)](https://docs.rs/cargo-cgp-expand)

# cargo-cgp-expand

**The readable expansions of `cargo cgp expand`, part of
[`cargo-cgp`](https://crates.io/crates/cargo-cgp), CGP's error toolchain.**

Context-Generic Programming (CGP) extends Rust with pluggable trait implementations, and its macros
generate the traits and impls that select them. `cargo cgp expand` shows that generated code, and
this crate makes it readable. The compiler prints CGP's type-level strings and lists in full, so a
field tag that you wrote as `Symbol!("name")` comes out as a nested type, with every name spelled
through the macros' own module (the impl's body is elided here):

```rust
impl ::cgp::macro_prelude::HasField<::cgp::macro_prelude::Symbol<4,
    ::cgp::macro_prelude::Chars<'n',
    ::cgp::macro_prelude::Chars<'a',
    ::cgp::macro_prelude::Chars<'m',
    ::cgp::macro_prelude::Chars<'e', ::cgp::macro_prelude::Nil>>>>>> for
    Person {
    // …
}
```

This crate rewrites such types back into the macros you write and drops those module paths, so
`cargo cgp expand` prints:

```rust
impl HasField<Symbol!("name")> for Person {
    // …
}
```

It also narrows an expansion to the one module, type, or trait that `cargo cgp expand --item`
names.

The crate links no compiler internals, so it builds and tests on any toolchain. It is published so
that `cargo-cgp-driver` can depend on it, and its API serves that driver alone, so it may change in
any release. To read CGP's generated code this way, install `cargo-cgp` instead.

The [CGP website](https://contextgeneric.dev/) explains how to
[expand CGP code](https://contextgeneric.dev/docs/tooling/cargo-cgp/expand), and teaches CGP itself.
