#![feature(prelude_import)]
//! An orphan registration beside `delegate_components!` on a foreign type, each with its own error.
//!
//! The registration keys `String` into a foreign component's per-type table, which is `[CGP-E022]`.
//! The wiring block's table is `Vec<u8>`, and a table is no namespace, though `DelegateComponent` has
//! a namespace's shape. Its impls are the program's own orphans, so they keep rustc's wording, and
//! they do not count as registrations that would make the caret ambiguous.
//!
//! CGP error class:
//! <https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/orphan-rule.md>.
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::component::DefaultImpls1;
use cgp::core::error::{ErrorRaiser, ErrorRaiserComponent};
use cgp::prelude::*;
impl<__Context__> ErrorRaiser<__Context__, String> for RaisePlain
where
    __Context__: HasErrorType,
{
    fn raise_error(error: String) -> <__Context__ as HasErrorType>::Error {
        {
            ::core::panicking::panic_fmt(format_args!("{0}", error));
        }
    }
}
impl<__Context__> IsProviderFor<ErrorRaiserComponent, __Context__, (String)>
for RaisePlain
where
    __Context__: HasErrorType,
{}
///Provider `RaisePlain` for [`ErrorRaiser`].
pub struct RaisePlain;
impl<__Components__> DefaultImpls1<ErrorRaiserComponent, __Components__> for String {
    type Delegate = RaisePlain;
}
impl DelegateComponent<ErrorRaiserComponent> for Vec<u8> {
    type Delegate = RaisePlain;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ErrorRaiserComponent, __Context__, __Params__> for Vec<u8>
where
    RaisePlain: IsProviderFor<ErrorRaiserComponent, __Context__, __Params__>,
{}
fn main() {}
