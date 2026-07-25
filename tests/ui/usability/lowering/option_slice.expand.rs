#![feature(prelude_import)]
//! Acceptable failure: a getter (or `#[implicit]` argument) typed `Option<&[T]>`
//! combines the `Option<&T>` and `&[T]` shorthands, a combination CGP does not
//! provide magic for. The shared `parse_field_type` lowers it literally by the
//! `Option<&T>` rule — reading an `Option<T>` field where `T` is the slice `[u8]`
//! — so the generated `HasField` bound names the unsized `Option<[u8]>`, which
//! `rustc` rejects (`[u8]` has no statically known size). CGP is working as
//! designed: the `Option` and slice shorthands ease the common single-shape
//! cases, and an unsupported combination is deferred to the compiler rather than
//! given a bespoke rule. The same boundary holds for `#[cgp_getter]` and for a
//! `#[cgp_fn]` `#[implicit]` argument, since all three share `parse_field_type`.
//!
//! See cgp-knowledge-base/cgp/errors/lowering/ill-formed-generated-type.md; the parser detail is in
//! cgp-knowledge-base/cgp/implementation/entrypoints/cgp_auto_getter.md (Behavior and corner
//! cases).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait HasItems {
    fn items(&self) -> Option<&[u8]>;
}
impl<__Context__> HasItems for __Context__
where
    __Context__: HasField<Symbol!("items"), Value = Option<[u8]>>,
{
    fn items(&self) -> Option<&[u8]> {
        self.get_field(::core::marker::PhantomData::<Symbol!("items")>).as_ref()
    }
}
fn main() {}
