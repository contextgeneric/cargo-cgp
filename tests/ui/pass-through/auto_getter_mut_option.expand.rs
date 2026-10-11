#![feature(prelude_import)]
//! A `&mut self` auto getter returning `Option<&String>`, which the generated body cannot produce.
//!
//! The error is a plain type mismatch, and rustc's suggestion is spliced after the attribute (a
//! known `cgp` defect). The tool must print what plain `cargo check` prints, short paths included
//! (`Option<&mut String>`, not `std::option::Option<&mut std::string::String>`).
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.
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
