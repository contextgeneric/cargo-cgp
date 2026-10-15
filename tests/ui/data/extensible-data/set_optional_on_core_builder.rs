//! Calling the optional builder's `set` on a core builder.
//!
//! `Context::builder().set(…)` fails on `Mapper == IsOptional`, since a core builder holds `foo`
//! unset rather than optional: ``[CGP-E050] field `foo` of `Context` is not optional``. The `help`
//! names the conversion with the import its trait needs, since rustc's own import suggestion names
//! a path that does not compile.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: CGP-E050 CGP-E104 CGP-E125

use cgp::extra::field::impls::SetOptional;
use cgp::prelude::*;

#[derive(CgpData)]
pub struct Context {
    pub foo: String,
    pub bar: u64,
}

fn main() {
    let _ = Context::builder().set(PhantomData::<Symbol!("foo")>, "foo".to_owned());
}
