//! A field transform with no conversion for one of the builder's markers.
//!
//! `OnlyAbsent` converts only absent fields, but `port` is already set. The output is an `E0277` on
//! `OnlyAbsent: TransformMap<IsPresent, IsPresent, u16>`, which names the field by its value type
//! rather than its name. rustc reports it twice, at the method name and at the whole call, and the
//! tool merges the repeat into the first block, whose labels already cover the call.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: none

use cgp::core::field::traits::{TransformMap, TransformMapFields};
use cgp::prelude::*;

pub struct OnlyAbsent;

impl<T: Default> TransformMap<IsNothing, IsPresent, T> for OnlyAbsent {
    fn transform_mapped(_value: ()) -> T {
        T::default()
    }
}

#[derive(CgpData)]
pub struct Config {
    pub port: u16,
    pub verbose: bool,
}

fn main() {
    let partial = Config::builder().build_field(PhantomData::<Symbol!("port")>, 8080);
    let _ = TransformMapFields::<OnlyAbsent, IsPresent>::transform_map_fields(partial).finalize_build();
}
