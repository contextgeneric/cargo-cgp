//! Running a command with a deadline, so a compiler that hangs fails its fixture instead of
//! stalling the run.

use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// How often the harness checks whether a command has ended.
const POLL: Duration = Duration::from_millis(20);

/// What one command printed and how it ended.
pub struct Captured {
    /// Its stdout.
    pub stdout: String,
    /// Its stderr.
    pub stderr: String,
    /// Whether it exited successfully; `false` when it timed out.
    pub success: bool,
    /// Whether it was killed for running past its deadline.
    pub timed_out: bool,
}

/// Run `command` to its end, or kill it once `timeout` has passed, and capture its output.
///
/// The command runs in a process group of its own, and a timeout kills the whole group: cargo
/// runs rustc as a child, and a rustc left running would keep the output pipes open, and the
/// worker's target directory busy, after cargo itself is gone.
///
/// Postcondition: `timed_out` iff the command was still running at `timeout`, and then `success`
/// is `false`.
pub fn capture(command: &mut Command, timeout: Duration) -> Captured {
    own_process_group(command);
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("starting {command:?}: {e}"));
    let stdout = read_to_end(child.stdout.take());
    let stderr = read_to_end(child.stderr.take());

    let deadline = Instant::now() + timeout;
    let status = loop {
        if let Some(status) = child.try_wait().expect("waiting for a command") {
            break Some(status);
        }
        if Instant::now() >= deadline {
            kill_group(&mut child);
            let _ = child.wait();
            break None;
        }
        thread::sleep(POLL);
    };

    Captured {
        stdout: stdout.join().expect("reading a command's stdout"),
        stderr: stderr.join().expect("reading a command's stderr"),
        success: status.is_some_and(|status| status.success()),
        timed_out: status.is_none(),
    }
}

/// Read a pipe to its end on a thread of its own, so a command that fills one pipe while the
/// harness waits cannot block on it.
fn read_to_end(pipe: Option<impl Read + Send + 'static>) -> JoinHandle<String> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut pipe) = pipe {
            let _ = pipe.read_to_end(&mut bytes);
        }
        String::from_utf8_lossy(&bytes).into_owned()
    })
}

#[cfg(unix)]
fn own_process_group(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
}

#[cfg(not(unix))]
fn own_process_group(_command: &mut Command) {}

/// Kill the child's process group, which [`own_process_group`] made its own, falling back to the
/// child alone.
fn kill_group(child: &mut Child) {
    #[cfg(unix)]
    {
        let group = format!("-{}", child.id());
        let killed = Command::new("kill")
            .args(["-KILL", "--", &group])
            .status()
            .is_ok_and(|status| status.success());
        if killed {
            return;
        }
    }
    let _ = child.kill();
}
