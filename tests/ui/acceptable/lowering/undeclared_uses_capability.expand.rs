#![feature(prelude_import)]
//! A `#[cgp_fn]` body that calls a capability method without declaring the capability via `#[uses]`.
//!
//! `#[cgp_fn]` (and `#[cgp_impl]`) turn a function into a blanket impl over a generated generic
//! context, `impl<__Context__> Describe for __Context__ where __Context__: GetName { … }`. The
//! body may call *other* CGP capabilities on `self`, but each such capability must be declared as a
//! dependency — with `#[uses(…)]` — so it becomes a `where` bound on `__Context__`. Here the body
//! calls `self.get_count()` but the `#[uses(GetName)]` list omits `GetCount`, so `__Context__` is
//! not bounded by `GetCount` and the call cannot resolve.
//!
//! Left to raw rustc this is a vague `E0599`: "the method `get_count` exists for reference
//! `&__Context__`, but its trait bounds were not satisfied", with a note about
//! `__Context__: HasField<Symbol!("count")>` — naming the generated `__Context__` the programmer
//! never wrote and pointing at the *wrong* fix (a missing field), when the real fix is to declare
//! the capability: `#[uses(GetName, GetCount)]`. The same shape arises for a forgotten CGP
//! *consumer* trait used the same way.
//!
//! The tool reshapes it into `[CGP-E012] the capability `GetCount` is used but not declared as a
//! dependency` with a `help: declare it as a dependency with `#[uses(GetCount)]``, recovered by
//! `resolve::detect_undeclared_capability`: the failing call sits in a generated blanket impl whose
//! `Self` is the bare `__Context__` parameter, its method belongs to the capability trait
//! `GetCount`, and that trait is not among the impl's `where` bounds. The `GetName` that *is*
//! declared is correctly left alone. (An async body's `[u8]: Sized` cascade — see the
//! money-transfer-shaped real case — is dropped by the same-line cascade suppression.)
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
/// A `#[cgp_fn]` capability that reads a `name` field.
pub trait GetName {
    fn get_name(&self) -> String;
}
/// A `#[cgp_fn]` capability that reads a `name` field.
impl<__Context__> GetName for __Context__
where
    Self: HasField<Symbol!("name"), Value = String>,
{
    fn get_name(&self) -> String {
        let name: &str = self
            .get_field(::core::marker::PhantomData::<Symbol!("name")>)
            .as_str();
        name.to_owned()
    }
}
/// A `#[cgp_fn]` capability that reads a `count` field.
pub trait GetCount {
    fn get_count(&self) -> u64;
}
/// A `#[cgp_fn]` capability that reads a `count` field.
impl<__Context__> GetCount for __Context__
where
    Self: HasField<Symbol!("count"), Value = u64>,
{
    fn get_count(&self) -> u64 {
        let count: &u64 = self
            .get_field(::core::marker::PhantomData::<Symbol!("count")>);
        *count
    }
}
/// A composite `#[cgp_fn]` that calls both capabilities, but declares only one: `GetCount` is used
/// in the body yet missing from `#[uses]`.
pub trait Describe {
    fn describe(&self) -> String;
}
/// A composite `#[cgp_fn]` that calls both capabilities, but declares only one: `GetCount` is used
/// in the body yet missing from `#[uses]`.
impl<__Context__> Describe for __Context__
where
    Self: GetName,
{
    fn describe(&self) -> String {
        ::alloc::__export::must_use({
            ::alloc::fmt::format(
                format_args!("{0} ({1})", self.get_name(), self.get_count()),
            )
        })
    }
}
fn main() {}
