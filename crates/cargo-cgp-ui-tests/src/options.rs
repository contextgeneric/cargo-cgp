//! Parsing the harness arguments.

/// Options controlling a suite run, parsed from the arguments cargo passes after `--`.
pub struct Options {
    /// Rewrite the snapshots from the current output instead of comparing.
    pub bless: bool,
    /// Print each fixture's raw output instead of comparing (interactive inspection).
    pub print: bool,
    /// How many fixtures to check concurrently, from `--jobs`/`-j`. `None` lets the
    /// harness pick a default from the machine's parallelism (see [`crate::runner`]).
    pub jobs: Option<usize>,
    /// Path substrings; a fixture runs only if its relative path contains one of them.
    /// Empty means run every fixture.
    pub filters: Vec<String>,
}

impl Options {
    /// Parse harness arguments. Recognized flags are `--bless`, `--print`, and
    /// `--jobs N` / `-j N` (also `--jobs=N`, `-jN`); any other `--flag` (such as those a
    /// test runner may inject) is ignored, and bare words become path filters.
    pub fn parse(args: impl IntoIterator<Item = String>) -> Self {
        let mut options = Options {
            bless: false,
            print: false,
            jobs: None,
            filters: Vec::new(),
        };

        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--bless" => options.bless = true,
                "--print" => options.print = true,
                // `--jobs`/`-j` take their count from the next argument, so consume it
                // even when it fails to parse — otherwise it would be read as a filter.
                "--jobs" | "-j" => options.jobs = args.next().and_then(|n| n.parse().ok()),
                flag if job_count(flag).is_some() => options.jobs = job_count(flag),
                flag if flag.starts_with('-') => {}
                _ => options.filters.push(arg),
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

/// The job count in a glued form — `--jobs=N`, `-j=N`, or `-jN` — or `None` if `flag` is
/// not such a form (or its count does not parse).
fn job_count(flag: &str) -> Option<usize> {
    let rest = flag
        .strip_prefix("--jobs=")
        .or_else(|| flag.strip_prefix("-j="))
        .or_else(|| flag.strip_prefix("-j"))?;
    rest.parse().ok()
}
