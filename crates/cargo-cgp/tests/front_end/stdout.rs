//! Tests for writing the front-end's answers to stdout, whose reader may have closed the pipe.

use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use cargo_cgp::stdout::write_output;

/// A writer whose every write fails with `kind`, as stdout does once its reader is gone.
struct FailingWriter(io::ErrorKind);

impl Write for FailingWriter {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(self.0.into())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Property: `write_output` fails exactly when the write fails for a reason other than a closed
/// pipe. Regression: `cargo cgp expand | head` on a long expansion ended with a broken-pipe error.
#[test]
fn only_a_closed_pipe_is_not_a_write_failure() {
    let mut written = Vec::new();
    write_output(&mut written, "text").expect("a working writer");
    assert_eq!(written, b"text");

    write_output(&mut FailingWriter(io::ErrorKind::BrokenPipe), "text")
        .expect("a closed pipe is not a failure");
    let error = write_output(&mut FailingWriter(io::ErrorKind::StorageFull), "text")
        .expect_err("a full disk is a failure");
    assert_eq!(error.kind(), io::ErrorKind::StorageFull);
}

/// A working directory under the system temp directory, unique to this process, removed on drop so
/// a failing assertion still cleans up.
struct WorkDir(PathBuf);

impl Drop for WorkDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A stdout whose reader is gone ends each of the front-end's answers quietly and successfully, as
/// `expand` already ends, rather than with `println!`'s panic. A user meets it by piping the help
/// into a reader that stops early, as `cargo cgp --help | head -1` does.
// Property: for each answer the front-end prints itself (the help, the version, and each
//   subcommand's help), a stdout closed before the front-end starts ends it with code 0 and no
//   panic on stderr.
// Reaches: the pipe's read end is closed before the front-end starts, so its first write fails;
//   none of these answers runs cargo, so the run needs no project.
#[cfg(unix)]
#[test]
fn a_closed_stdout_ends_each_answer_quietly() {
    use std::process::{Command, Stdio};

    let dir = WorkDir(
        std::env::temp_dir().join(format!("cargo-cgp-front-end-stdout-{}", std::process::id())),
    );
    fs::create_dir_all(&dir.0).expect("creating the working directory");

    for args in [
        &["--help"][..],
        &["--version"][..],
        &[][..],
        &["expand", "--help"][..],
        &["setup", "--help"][..],
        &["update", "--help"][..],
    ] {
        let (reader, writer) = io::pipe().expect("creating a pipe");
        drop(reader);
        let output = Command::new(env!("CARGO_BIN_EXE_cargo-cgp"))
            .args(args)
            .stdout(writer)
            .stderr(Stdio::piped())
            .current_dir(&dir.0)
            .output()
            .expect("running the front-end");
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert_eq!(output.status.code(), Some(0), "{args:?}: {stderr}");
        assert!(!stderr.contains("panicked"), "{args:?}: {stderr}");
    }
}
