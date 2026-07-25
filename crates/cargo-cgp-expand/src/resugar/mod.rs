//! The resugaring passes, one module per construct.

pub mod file;
pub mod list;
pub mod parts;
pub mod path;
pub mod spacing;
pub mod strip;
pub mod symbol;

pub use file::*;
pub use list::*;
pub use path::*;
pub use spacing::*;
pub use strip::*;
pub use symbol::*;
