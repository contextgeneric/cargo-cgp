//! Calls named like CGP's extensible-data operations, on a type of a module that does not import
//! them.
//!
//! `Config::builder()` and `config.set(80)` read like a CGP builder's `builder` and an optional
//! builder's `set`, but the module brings neither `HasBuilder` nor `SetOptional` into scope, so the
//! compiler never considered them and the calls fail as plain missing items. The tool must print
//! what plain `cargo check` prints, rather than report that `Config` lacks a data derive.
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.

use cgp::prelude::*;

#[derive(HasField)]
pub struct App {
    pub name: String,
}

mod plain {
    pub struct Config {
        pub port: u16,
    }

    pub fn make() -> Config {
        Config::builder()
    }

    pub fn update(config: Config) {
        config.set(80);
    }
}

fn main() {
    plain::update(plain::make());
}
