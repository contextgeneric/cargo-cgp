//! Thin entrypoint for the `cargo-cgp` subcommand. All logic lives in the library's
//! [`cargo_cgp::run::run`]; this wrapper only translates its result into an exit code.

use std::process;

fn main() {
    match cargo_cgp::run::run() {
        Ok(code) => process::exit(code),
        Err(error) => {
            eprintln!("cargo-cgp: {error:#}");
            process::exit(1);
        }
    }
}
