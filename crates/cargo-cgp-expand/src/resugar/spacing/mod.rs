//! Tightening the spacing the printer leaves inside a resugared macro call, one module per
//! concern: the pass, the scan that finds the macro bodies, and the two layouts.

mod layout;
mod scan;
mod tighten_bodies;

pub use layout::*;
pub use scan::*;
pub use tighten_bodies::*;
