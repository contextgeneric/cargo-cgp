//! The `cargo-cgp-driver` rustc wrapper.
//!
//! This crate is the `rustc_driver`-based compiler wrapper that `cargo-cgp` sets as the
//! `RUSTC_WORKSPACE_WRAPPER`, so cargo invokes it in place of `rustc` for every workspace
//! crate. It mirrors `clippy-driver`: cargo calls it as
//! `cargo-cgp-driver <path-to-rustc> <rustc args...>`, and we run the real compiler
//! in-process through [`rustc_driver`] with our own [`callbacks::CgpCallbacks`].
//!
//! Besides injecting the diagnostic flags (see [`args`]), the callbacks install a custom
//! diagnostic [`emitter`] that queries the compiler to name the consumer and provider
//! traits behind a CGP component marker and rewrites the compiler's wiring notes
//! accordingly. Aside from those transformations the driver compiles exactly as `rustc`
//! would. Routing through `rustc_driver` is what gives the emitter access to the live
//! `TyCtxt` needed to recover those names.
//!
//! # `rustc_private`
//!
//! The compiler's internal libraries are unstable, so linking them requires the
//! `rustc_private` feature and a nightly toolchain with the `rustc-dev` component. The
//! `extern crate` declarations below pull those libraries from the sysroot; add another
//! `extern crate rustc_*;` line here when a module needs a further compiler crate.
//!
//! The entrypoint is [`run::run`], invoked by the thin `bin/cargo-cgp-driver.rs` wrapper.

#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_infer;
extern crate rustc_interface;
extern crate rustc_lint_defs;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_trait_selection;

pub mod args;
pub mod callbacks;
pub mod component_map;
pub mod config;
pub mod emitter;
pub mod help;
pub mod resolve;
pub mod run;
pub mod version;
