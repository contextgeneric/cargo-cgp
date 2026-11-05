//! A type mismatch in a provider body, in a crate that also holds an unrelated `E0107`.
//!
//! `LoadOrFail` returns a `String` where the context's abstract `Error` belongs (`E0308`), and an
//! unrelated function names `Vec` without its argument (`E0107`). Both are plain Rust errors, and
//! neither is a `#[cgp_impl]` header mistake, so the tool must print both exactly as rustc does:
//! the `E0107` elsewhere in the crate must not make the body's own error disappear.
//!
//! CGP error class: none; plain Rust the tool passes through.

use cgp::prelude::*;

#[cgp_component(Loader)]
#[use_type(HasErrorType.Error)]
pub trait CanLoad {
    fn load(&self) -> Result<u32, Error>;
}

// MISTAKE: the body returns a `String` where the abstract `Error` belongs.
#[cgp_impl(new LoadOrFail)]
#[uses(CanRaiseError<String>)]
#[use_type(HasErrorType.Error)]
impl Loader {
    fn load(&self) -> Result<u32, Error> {
        Err("failed".to_owned())
    }
}

// MISTAKE: `Vec` without its element type, unrelated to the provider above.
pub fn unrelated(_: Vec) {}

fn main() {}
