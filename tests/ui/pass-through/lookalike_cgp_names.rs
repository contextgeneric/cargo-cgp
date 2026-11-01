//! A program with its own `Symbol`, `Chars`, `Nil`, and `HasField`, unrelated to CGP's.
//!
//! The program never uses `cgp`, so its unmet `HasField` bound is plain Rust, and the tool must not
//! resugar the bound or reword its help into a `#[derive(HasField)]` hint. It still differs from
//! plain `cargo check`, a documented limitation: the injected `--verbose` prints
//! `Symbol<1, Chars<'a', Nil>>` in full where rustc elides it to `Symbol<1, Chars<..>>`.
//!
//! CGP error class: none; plain Rust the tool passes through.
//!
//! Limitation: cgp-knowledge-base/cargo-cgp/implementation/driver/program.md, "Un-eliding the diagnostic".

//@rewrites-falsely: the injected `--verbose` prints types rustc elides (a documented limitation)

use std::marker::PhantomData;

pub struct Chars<const C: char, T>(PhantomData<T>);
pub struct Nil;
pub struct Symbol<const N: usize, T>(PhantomData<T>);

pub trait HasField<Tag> {
    type Value;
}

pub struct Ctx;

impl HasField<Symbol<1, Chars<'b', Nil>>> for Ctx {
    type Value = u8;
}

fn need<T: HasField<Symbol<1, Chars<'a', Nil>>>>() {}

fn main() {
    need::<Ctx>();
}
