//! The `expand` subcommand: showing the Rust a project's CGP macros generate.

mod command;
mod item;
mod output;
mod profile;

pub use command::*;
pub use item::*;
pub use output::*;
pub use profile::*;
