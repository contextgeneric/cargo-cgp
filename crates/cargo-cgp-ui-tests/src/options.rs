//! Parsing the harness arguments.

/// Options controlling a suite run, parsed from the arguments cargo passes after `--`.
pub struct Options {
    /// Rewrite the `.stderr` snapshots from the current output instead of comparing.
    pub bless: bool,
    /// Print each fixture's raw output instead of comparing (interactive inspection).
    pub print: bool,
    /// Path substrings; a fixture runs only if its relative path contains one of them.
    /// Empty means run every fixture.
    pub filters: Vec<String>,
}

impl Options {
    /// Parse harness arguments. Recognized flags are `--bless` and `--print`; any other
    /// `--flag` (such as those a test runner may inject) is ignored, and bare words
    /// become path filters.
    pub fn parse(args: impl IntoIterator<Item = String>) -> Self {
        let mut options = Options {
            bless: false,
            print: false,
            filters: Vec::new(),
        };

        for arg in args {
            match arg.as_str() {
                "--bless" => options.bless = true,
                "--print" => options.print = true,
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
