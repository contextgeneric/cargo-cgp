# AGENTS.md — cargo-cgp (front-end)

This crate is the `cargo-cgp` cargo-subcommand front-end. Read the workspace
[../../AGENTS.md](../../AGENTS.md) first for the two-binary architecture and the project's goals;
this file covers only what is specific to the front-end.

The front-end's whole job is to launch `cargo check` with the driver wired in as the workspace rustc
wrapper. It is a plain `std` + `anyhow` binary and **must not** depend on `rustc_private` or on the
driver crate as a library — it finds the driver executable on disk at runtime, as a sibling of
itself, exactly as `cargo-clippy` finds `clippy-driver`. Keeping compiler internals out of this
crate is deliberate; do not pull them in.

The entrypoint is [`run::run`](src/run.rs), called by the thin [`bin/cargo-cgp.rs`](bin/cargo-cgp.rs)
wrapper. It normalizes arguments with [`args`](src/args.rs) — stripping the `cgp` token cargo
inserts, so `cargo cgp check` and a direct `cargo-cgp check` reduce to the same thing — then
dispatches on the subcommand. The only subcommand is `check`, implemented under
[`check/`](src/check): [`command.rs`](src/check/command.rs) builds and runs the wrapped `cargo
check`, setting `RUSTC_WORKSPACE_WRAPPER`, passing the sysroot to the driver through
`CARGO_CGP_SYSROOT`, and prepending the sysroot `lib` directory to the dynamic-library search path;
[`driver_path.rs`](src/check/driver_path.rs) locates the sibling driver; and
[`sysroot.rs`](src/check/sysroot.rs) discovers the toolchain sysroot via `rustc --print sysroot`.
The well-known strings all three need live in [`config.rs`](src/config.rs) and are passed in as
parameters.

When you add a subcommand, add its handler as a sibling of `check`, dispatch to it from `run`, and
keep the `bin` wrapper untouched. Cover argument handling with unit tests as `args.rs` does.
