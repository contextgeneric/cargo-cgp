//! Locating the workspace, the fixtures, the cgp checkout, and the built binaries.
//!
//! The harness is a separate crate, so it cannot use `CARGO_BIN_EXE_*` (those name only
//! the binaries of the crate under test). Instead it finds the `target/debug` directory
//! by walking up from its own executable, which is robust to `CARGO_TARGET_DIR` and to
//! where cargo chooses to place a test binary, and resolves everything else relative to
//! the workspace root.

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The cargo-cgp workspace root — two levels up from this crate's manifest
/// (`crates/cargo-cgp-ui-tests`).
pub fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolving the workspace root")
}

/// The UI fixture tree, `tests/ui/` under the workspace root.
pub fn fixtures_dir() -> PathBuf {
    workspace_root().join("tests/ui")
}

/// The stored auxiliary-crate sources a fixture can depend on via a
/// `//@aux-build:` directive (`crates/cargo-cgp-ui-tests/auxiliary`). Each holds a
/// `src/` tree and a manifest template whose `cgp` path is filled in at build time.
pub fn auxiliary_src_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("auxiliary")
}

/// Where auxiliary crates are materialized for a run (`target/ui-harness/aux`),
/// as siblings so an aux crate's `../other-aux` path dependency resolves. Computed
/// without creating it.
pub fn aux_build_root() -> PathBuf {
    harness_crate_root().join("aux")
}

/// The cgp facade crate, assumed to live in a sibling `cgp` checkout at `../cgp`.
pub fn cgp_crate_dir() -> PathBuf {
    workspace_root()
        .join("../cgp/crates/main/cgp")
        .canonicalize()
        .expect("resolving the cgp crate (expected a sibling ../cgp checkout)")
}

/// The root of the cgp checkout (`crates/main/cgp` stripped from [`cgp_crate_dir`]). Its
/// absolute path appears in cross-crate diagnostic notes, so snapshots normalize it away.
pub fn cgp_root() -> PathBuf {
    cgp_crate_dir()
        .ancestors()
        .nth(3)
        .expect("deriving the cgp checkout root")
        .to_path_buf()
}

/// The `target/debug` directory, found by walking up from the harness test binary until
/// an ancestor holds the built `cargo-cgp` front-end. Searching for the binary rather
/// than counting parent directories keeps this independent of where cargo puts a test
/// executable, which differs between cargo versions (`<target>/debug/deps/ui-<hash>` in
/// some, `<target>/debug/build/<package>/<hash>/out/ui-<hash>` in others).
pub fn debug_dir() -> PathBuf {
    let exe = env::current_exe().expect("resolving the harness executable path");
    let front_end = front_end_name();
    ancestor_holding(&exe, &front_end).unwrap_or_else(|| {
        panic!(
            "deriving target/debug from the harness executable {}: no ancestor holds a \
             built `{front_end}` — run `cargo build` first",
            exe.display(),
        )
    })
}

/// The nearest ancestor directory of `from` that directly contains a file named `binary`,
/// searching upwards. `from` itself is considered, so passing a directory finds `binary`
/// in that directory. This is the layout-independent half of [`debug_dir`], kept a plain
/// function of its arguments so it can be exercised against either cargo layout.
pub fn ancestor_holding(from: &Path, binary: &str) -> Option<PathBuf> {
    from.ancestors()
        .find(|dir| dir.join(binary).is_file())
        .map(Path::to_path_buf)
}

/// The `cargo-cgp` front-end's file name, with the platform's executable suffix.
fn front_end_name() -> String {
    format!("cargo-cgp{}", env::consts::EXE_SUFFIX)
}

/// The sysroot of the toolchain the harness runs under, from `rustc --print sysroot`.
///
/// A diagnostic that points into the standard library — an implicit `Sized` bound on
/// `Option`, say — names a path inside this directory, which carries the user's home
/// directory, the pinned nightly's name, and the host target triple. The snapshots
/// normalize it to `$SYSROOT` for the same reason they normalize the `cgp` checkout.
pub fn sysroot() -> PathBuf {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    // Ask through `rustc` as cargo resolves it, so this is the toolchain the fixtures are
    // actually compiled with rather than whatever `rustc` is first on `PATH`.
    let rustc = Path::new(&cargo).with_file_name(format!("rustc{}", env::consts::EXE_SUFFIX));
    let output = Command::new(if rustc.is_file() {
        rustc.into_os_string()
    } else {
        "rustc".into()
    })
    .args(["--print", "sysroot"])
    .output()
    .expect("running `rustc --print sysroot`");

    PathBuf::from(
        String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_owned(),
    )
}

/// The root under which the per-worker throwaway crates live (`target/ui-harness`),
/// computed without creating it.
pub fn harness_crate_root() -> PathBuf {
    debug_dir()
        .parent()
        .expect("target directory")
        .join("ui-harness")
}

/// The directory of throwaway crate number `index` (`target/ui-harness/worker-<index>`),
/// computed without creating it. Each concurrent worker owns one such crate so that
/// running fixtures in parallel never contend on a shared `src/main.rs` or cargo target
/// lock; see [`crate::runner`].
pub fn worker_crate_dir(index: usize) -> PathBuf {
    harness_crate_root().join(format!("worker-{index}"))
}

/// Path to the built `cargo-cgp` front-end binary.
pub fn cargo_cgp_bin() -> PathBuf {
    debug_dir().join(front_end_name())
}
