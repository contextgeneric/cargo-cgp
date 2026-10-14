//! A plain generic function bounded on a variant the enum lacks, and the shredded spine in its help.
//!
//! `needs_bar::<Big>()` requires `Big: FromVariant<Symbol!("Bar")>`, and `Big` has no `Bar` variant.
//! No anchor recovers a plain function's bound, so the diagnostic keeps rustc's headline, which
//! names the variant. It pins the post-processing a declined diagnostic still gets: rustc builds its
//! "similar impl" help out of styled fragments split at every difference between the two traits,
//! which shreds `Symbol<3, Chars<'B', …>>` into a fragment per character; the fragments are read as
//! the one line they render as, so the help says `Symbol!("Foo")`, as the headline says
//! `Symbol!("Bar")`.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: none

use cgp::prelude::*;

#[derive(CgpData)]
pub enum Big {
    Foo(u64),
}

fn needs_bar<T: FromVariant<Symbol!("Bar")>>() {}

fn main() {
    needs_bar::<Big>();
}
