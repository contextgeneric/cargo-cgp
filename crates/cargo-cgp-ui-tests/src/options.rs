//! Parsing the harness arguments.

use std::num::NonZeroUsize;
use std::time::Duration;

/// How long one command may run before the harness kills it and fails the fixture, unless
/// `--timeout` says otherwise. A worker's first command also builds `cgp`, and under threads each
/// of up to eight workers runs a multi-threaded rustc, so the limit leaves room for both.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(600);

/// Options controlling a suite run, parsed from the arguments cargo passes after `--`.
pub struct Options {
    /// Rewrite the snapshots from the current output instead of comparing.
    pub bless: bool,
    /// Print each fixture's raw output instead of comparing (interactive inspection).
    pub print: bool,
    /// List the selected fixtures with their declared quality and codes, compiling nothing.
    pub list: bool,
    /// How many fixtures to check concurrently, from `--jobs`/`-j`. `None` lets the
    /// harness pick a default from the machine's parallelism (see [`crate::runner`]).
    pub jobs: Option<usize>,
    /// How many threads the tool's passes run rustc's front end on, from `--threads`.
    pub threads: Threads,
    /// How many times each fixture's passes run, from `--iterations`; the first iteration
    /// with a failure ends the fixture's runs.
    pub iterations: NonZeroUsize,
    /// How long one command may run, from `--timeout` in seconds.
    pub timeout: Duration,
    /// Path substrings; a fixture runs only if its relative path contains one of them.
    /// Empty means run every fixture.
    pub filters: Vec<String>,
}

/// The front end's thread count for the tool's passes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Threads {
    /// `-Zthreads=N`, from `--threads N`.
    Count(NonZeroUsize),
    /// No thread count, so rustc uses its built-in default as a release run does, from
    /// `--threads default`.
    Default,
}

impl Threads {
    /// One thread, the mode when `--threads` is absent.
    pub const ONE: Threads = Threads::Count(NonZeroUsize::MIN);

    /// The flag the tool's passes add to `RUSTFLAGS`, or `None` for rustc's default.
    pub fn flag(self) -> Option<String> {
        match self {
            Threads::Count(count) => Some(format!("-Zthreads={count}")),
            Threads::Default => None,
        }
    }

    /// Whether the front end may run on more than one thread.
    pub fn is_parallel(self) -> bool {
        self != Threads::ONE
    }
}

impl Options {
    /// Parse harness arguments. Recognized flags are `--bless`, `--print`, `--list`,
    /// `--jobs N` / `-j N` (also `--jobs=N`, `-jN`), `--threads N|default`, `--iterations K`, and
    /// `--timeout SECS` (each of the last three also as `--name=value`); any other `--flag` (such
    /// as those a test runner may inject) is ignored, and bare words become path filters.
    ///
    /// # Panics
    ///
    /// When `--threads`, `--iterations`, or `--timeout` lacks a value or has one that does not
    /// parse, since falling back to the default would run another mode than the one asked for.
    pub fn parse(args: impl IntoIterator<Item = String>) -> Self {
        let mut options = Options {
            bless: false,
            print: false,
            list: false,
            jobs: None,
            threads: Threads::ONE,
            iterations: NonZeroUsize::MIN,
            timeout: DEFAULT_TIMEOUT,
            filters: Vec::new(),
        };

        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            let (name, glued) = match arg.split_once('=') {
                Some((name, value)) => (name, Some(value.to_owned())),
                None => (arg.as_str(), None),
            };
            let mut value = || glued.clone().or_else(|| args.next());
            match name {
                "--threads" => options.threads = parse_threads(value()),
                "--iterations" => options.iterations = parse_positive("--iterations", value()),
                "--timeout" => {
                    let seconds = parse_positive("--timeout", value());
                    options.timeout = Duration::from_secs(seconds.get() as u64);
                }
                _ => match arg.as_str() {
                    "--bless" => options.bless = true,
                    "--print" => options.print = true,
                    "--list" => options.list = true,
                    // `--jobs`/`-j` take their count from the next argument, so consume it
                    // even when it fails to parse, since otherwise it would be read as a filter.
                    "--jobs" | "-j" => options.jobs = args.next().and_then(|n| n.parse().ok()),
                    flag if job_count(flag).is_some() => options.jobs = job_count(flag),
                    flag if flag.starts_with('-') => {}
                    _ => options.filters.push(arg),
                },
            }
        }

        options
    }

    /// Whether a fixture with the given relative path should run under these filters.
    pub fn matches(&self, relative_path: &str) -> bool {
        self.filters.is_empty()
            || self
                .filters
                .iter()
                .any(|filter| relative_path.contains(filter))
    }
}

/// The job count in a glued form (`--jobs=N`, `-j=N`, or `-jN`) or `None` if `flag` is
/// not such a form (or its count does not parse).
fn job_count(flag: &str) -> Option<usize> {
    let rest = flag
        .strip_prefix("--jobs=")
        .or_else(|| flag.strip_prefix("-j="))
        .or_else(|| flag.strip_prefix("-j"))?;
    rest.parse().ok()
}

/// `--threads`'s value: `default`, or a thread count of at least one.
fn parse_threads(value: Option<String>) -> Threads {
    match value.as_deref() {
        Some("default") => Threads::Default,
        _ => Threads::Count(parse_positive("--threads", value)),
    }
}

/// A flag's value as a positive count, panicking with the flag's name otherwise.
fn parse_positive(flag: &str, value: Option<String>) -> NonZeroUsize {
    value
        .as_deref()
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(|| panic!("`{flag}` takes a positive number, got {value:?}"))
}
