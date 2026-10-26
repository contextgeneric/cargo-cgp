//! A data operation that takes no `self`, called as a method on a type without the data derives.
//!
//! `HasBuilder::builder` takes no `self`, so the compiler reports `config.builder()` as an
//! associated function called as a method. The data-call anchor considers only operations taking
//! `self` for a method call, so it does not read the call as a builder of a type without the
//! derive, whose fix would leave the call still wrong. The tool must print what plain
//! `cargo check` prints.
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.

use cgp::prelude::*;

pub struct Config {
    pub port: u16,
}

#[derive(HasField)]
pub struct App {
    pub name: String,
}

fn main() {
    let config = Config { port: 80 };
    let _ = config.builder();
}
