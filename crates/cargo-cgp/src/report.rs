//! How the front-end words what it says about itself, in cargo's style.
//!
//! cargo reports a failure as `error: …` with each cause under its own `Caused by:`, a warning as
//! `warning: …`, advice as `help: …`, and progress as a right-aligned status word before the
//! message, all on stderr. The front-end's own messages follow the same forms, so its output reads
//! as part of cargo's. Each form is a pure function over the text, so the shapes are unit-tested.

/// The width cargo right-aligns a status word to.
const STATUS_WIDTH: usize = 12;

/// The report for an error the front-end stops on: `error:` and its message, then each cause the
/// error was given as context for, under a `Caused by:` of its own and indented, as cargo prints a
/// cause chain.
pub fn error_report(error: &anyhow::Error) -> String {
    let mut report = format!("error: {error}");
    for cause in error.chain().skip(1) {
        report.push_str("\n\nCaused by:");
        report.push_str(&indented(&cause.to_string()));
    }
    report
}

/// A warning: something the user set is not used as they may expect, and the run goes on.
pub fn warning(message: &str) -> String {
    format!("warning: {message}")
}

/// A progress line: `verb` right-aligned, as cargo prints `Compiling` or `Installing`.
pub fn status(verb: &str, message: &str) -> String {
    format!("{verb:>STATUS_WIDTH$} {message}")
}

/// What another program printed, quoted as a block to append to a message: a colon, a blank line,
/// and the text indented two spaces, so a line such as rustup's own `error: …` reads as part of the
/// quote rather than as a second error. Nothing when the text is blank.
pub fn quoted_output(text: &str) -> String {
    let text = text.trim();
    if text.is_empty() {
        String::new()
    } else {
        format!(":\n{}", indented(text))
    }
}

/// `text` with every non-empty line indented two spaces, each on a line of its own.
fn indented(text: &str) -> String {
    text.lines()
        .map(|line| {
            if line.is_empty() {
                "\n".to_owned()
            } else {
                format!("\n  {line}")
            }
        })
        .collect()
}

/// The `help` that ends every failure `cargo cgp setup` repairs.
pub const RUN_SETUP_HELP: &str = "help: run `cargo cgp setup`";

/// The `help` for a machine without rustup, which both the check and `setup` need.
pub const INSTALL_RUSTUP_HELP: &str =
    "help: install rustup, or install cargo-cgp through its Nix flake, which needs no rustup";
