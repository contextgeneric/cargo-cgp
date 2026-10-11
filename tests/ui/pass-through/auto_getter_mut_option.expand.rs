#![feature(prelude_import)]
//! A `&mut self` auto getter returning `Option<&mut String>`, a known `cgp` defect.
//!
//! rustc's suggestion is spliced after the attribute, and the tool prints it with full paths
//! (`std::option::Option<&mut std::string::String>`).
//!
//! CGP error class: none; a driver path-printing defect affecting untransformed diagnostics.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
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
