//! A provider for a component with `#[extend(TagFor<u32>)]` that omits `#[uses(TagFor<u32>)]`.
//!
//! The component requires the generic trait `TagFor<u32>` of every context, so its provider trait
//! carries the bound, and a provider that does not declare it fails with `E0277` on
//! `__Context__: TagFor<u32>` at the `#[cgp_impl]` attribute. rustc's help names the `HasField`
//! bound inside `TagFor`, not the missing declaration.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[cgp_fn]
pub fn tag_for<T>(&self, #[implicit] count: &u64) -> u64 {
    let _ = core::marker::PhantomData::<T>;
    *count
}

#[cgp_component(Tagger)]
#[extend(TagFor<u32>)]
pub trait CanTag {
    fn tag(&self) -> u64;
}

#[cgp_impl(new TagIt)]
impl Tagger {
    fn tag(&self) -> u64 {
        0
    }
}

fn main() {}
