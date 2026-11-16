//! The exit code the front-end reports for a child process it ran.

use std::process::ExitStatus;

/// The exit code to propagate for a child that ended with `status`: its own code, `128 + signal` on
/// Unix when a signal ended it, and 1 otherwise; see
/// `cgp-knowledge-base/cargo-cgp/implementation/executable-structure.md`.
pub fn exit_code(status: ExitStatus) -> i32 {
    if let Some(code) = status.code() {
        return code;
    }
    #[cfg(unix)]
    if let Some(signal) = std::os::unix::process::ExitStatusExt::signal(&status) {
        return 128 + signal;
    }
    1
}
