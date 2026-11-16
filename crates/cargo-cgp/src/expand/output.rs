//! Where the driver writes the expansion, and how the front-end reads it back.

use std::fs::{self, DirBuilder};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::{env, process};

/// How many names [`ExpansionDir::create_in`] tries before giving up, when other directories
/// already hold the earlier ones.
const NAME_ATTEMPTS: u32 = 100;

/// A directory only this user can enter, created for one expansion and removed when dropped; see
/// `cgp-knowledge-base/cargo-cgp/implementation/expand-command.md`.
pub struct ExpansionDir {
    dir: PathBuf,
}

impl ExpansionDir {
    /// Create the directory under the system temp directory.
    pub fn create() -> io::Result<Self> {
        Self::create_in(&env::temp_dir())
    }

    /// Create the directory under `parent`. The name carries the process id and a counter, and the
    /// directory is created exclusively, so a name another process already holds is skipped rather
    /// than reused.
    pub fn create_in(parent: &Path) -> io::Result<Self> {
        let mut builder = DirBuilder::new();
        #[cfg(unix)]
        std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);

        for attempt in 0..NAME_ATTEMPTS {
            let dir = parent.join(format!("cargo-cgp-expand-{}-{attempt}", process::id()));
            match builder.create(&dir) {
                Ok(()) => return Ok(Self { dir }),
                Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(
            ErrorKind::AlreadyExists,
            format!(
                "every directory name tried under {} is taken",
                parent.display()
            ),
        ))
    }

    /// The directory itself.
    pub fn path(&self) -> &Path {
        &self.dir
    }

    /// The file the driver writes the finished expansion to.
    pub fn output_path(&self) -> PathBuf {
        self.dir.join("expansion.rs")
    }
}

impl Drop for ExpansionDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

/// Read the expansion the driver wrote, or `None` when it wrote nothing; the directory is new for
/// each run, so a stale file cannot be mistaken for output.
pub fn read_expansion(path: &Path) -> Option<String> {
    let expansion = fs::read_to_string(path).ok()?;
    (!expansion.trim().is_empty()).then_some(expansion)
}
