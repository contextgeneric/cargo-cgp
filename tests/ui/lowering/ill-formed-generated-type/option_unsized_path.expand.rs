#![feature(prelude_import)]
//! A getter typed `Option<&Path>`, an optional reference to an unsized type named by a path.
//!
//! The `Option<&T>` form reads an `Option<T>` field, and `Path` is unsized, so the generated
//! `HasField` bound names the ill-formed `Option<Path>`, and the compiler rejects it with the
//! `E0277` `Sized` form on the `#[cgp_auto_getter]` attribute, its headline naming `[u8]`, the
//! slice `Path` wraps, and a note naming `Path`. The macro rejects an optional reference to a slice
//! or a trait object, whose shape it can see, but a path names a type whose size only the compiler
//! knows, so this one reaches the compiler. The fix is to read the sized owner, `Option<&PathBuf>`
//! from an `Option<PathBuf>` field.
//!
//! CGP error class: ill-formed generated type (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/ill-formed-generated-type.md).
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use std::path::Path;
use cgp::prelude::*;
pub trait HasConfigPath {
    fn config_path(&self) -> Option<&Path>;
}
impl<__Context__> HasConfigPath for __Context__
where
    __Context__: HasField<Symbol!("config_path"), Value = Option<Path>>,
{
    fn config_path(&self) -> Option<&Path> {
        ::core::option::Option::as_ref(
            HasField::get_field(
                self,
                ::core::marker::PhantomData::<Symbol!("config_path")>,
            ),
        )
    }
}
fn main() {}
