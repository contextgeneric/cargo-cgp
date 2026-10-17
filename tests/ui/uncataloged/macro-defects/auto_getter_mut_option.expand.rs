#![feature(prelude_import)]
//! A `&mut self` auto getter returning `Option<&String>`, which the generated body cannot produce.
//!
//! The error is a plain type mismatch, and rustc's suggestion is spliced after the attribute (a
//! known `cgp` defect), where no edit of the source can apply it; the tool drops it. What remains is
//! the mismatch with short paths (`Option<&mut String>`, not
//! `std::option::Option<&mut std::string::String>`).
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait HasNickname {
    fn nickname(&mut self) -> Option<&String>;
}
impl<__Context__> HasNickname for __Context__
where
    __Context__: HasFieldMut<Symbol!("nickname"), Value = Option<String>>,
{
    fn nickname(&mut self) -> Option<&String> {
        self.get_field_mut(::core::marker::PhantomData::<Symbol!("nickname")>).as_mut()
    }
}
fn main() {}
