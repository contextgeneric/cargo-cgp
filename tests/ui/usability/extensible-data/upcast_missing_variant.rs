//! An extensible-data cast the tool does not reshape, and the shredded spine in its help.
//!
//! `CanUpcast` widens one enum into another that must cover every variant of it. `Small`
//! has a `Bar` variant that `Big` lacks, so the cast cannot hold, and the mistake a reader
//! needs stated is exactly that: *`Small`'s `Bar` variant has no counterpart in `Big`*.
//!
//! The typed resolver declines the whole extensible-data family — casts, builders,
//! extractors — so this passes through as rustc wrote it. The reader is left with an
//! internal `FromVariant` bound, a caret on the *wrong* variant (`Big`'s `Foo`, the one
//! that does match), the macro-generated `__PartialSmall<IsVoid, IsPresent>` extractor
//! state they never wrote, and a hidden requirement. Reshaping this class into a coded
//! leaf naming the absent variant is the work outstanding.
//!
//! What the fixture *does* pin is the post-processing that survives a decline. rustc builds
//! its "similar impl" help out of styled fragments split at every difference between the two
//! traits, which shreds `Symbol<3, Chars<'B', …>>` into a fragment per character; read
//! fragment by fragment no CGP construct matches, so the help used to show the raw spine
//! while the main message beside it read `Symbol!("Bar")`. The fragments are now read as the
//! one line they render as, so both say `Symbol!("Bar")`.
//!
//! CGP error class: none yet — the extensible-data casts
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/extensible-variants.md)
//! are not in the upstream catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

use cgp::core::field::impls::CanUpcast;
use cgp::prelude::*;

#[derive(CgpData)]
pub enum Small {
    Foo(u64),
    Bar(String),
}

// `Big` is missing `Small`'s `Bar` variant, so it cannot receive every value of `Small`.
#[derive(CgpData)]
pub enum Big {
    Foo(u64),
}

fn main() {
    let _: Big = Small::Foo(1).upcast(PhantomData::<Big>);
}
