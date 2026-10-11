//! A field transform with no conversion for one of the builder's markers.
//!
//! `OnlyAbsent` converts only absent fields, but `port` is already set. The output is the same
//! `E0277` on `OnlyAbsent: TransformMap<IsPresent, IsPresent, u16>` twice, at two carets, with no
//! de-duplication, and it names the field by its value type rather than its name.
//!
//! CGP error class: none yet; the extensible-data family
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/extensible-records.md)
//! is not in the upstream catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
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
