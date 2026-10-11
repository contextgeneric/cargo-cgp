//! The exit code the front-end reports for a child process it ran.

use std::process::ExitStatus;

/// The exit code to propagate for a child that ended with `status`.
///
/// A child that exited passes its code through. On Unix, a child a signal ended (an interrupted
/// `cargo` run, say) has no exit code, so the front-end reports `128 + signal`, the code a shell
/// reports for it, rather than a plain failure. Elsewhere, and for a status that is neither, it is 1.
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
