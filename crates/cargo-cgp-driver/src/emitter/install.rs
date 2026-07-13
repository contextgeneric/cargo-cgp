//! Installing the transforming emitter on the compiler session.
//!
//! The session's own emitter cannot be *wrapped* — [`set_emitter`](rustc_errors::DiagCtxt::set_emitter)
//! only replaces it, with no way to recover the original — so [`install`] rebuilds the same inner
//! emitter the compiler's `default_emitter` would build for the active error format (a
//! [`JsonEmitter`] or an [`AnnotateSnippetEmitter`]) and wraps *that* in
//! [`CgpEmitter`](super::CgpEmitter), so `cargo-cgp-driver` renders like vanilla `rustc` in either
//! format, only with the CGP transforms applied.

use std::io;

use rustc_errors::TerminalUrl;
use rustc_errors::annotate_snippet_emitter_writer::AnnotateSnippetEmitter;
use rustc_errors::emitter::{HumanReadableErrorType, OutputTheme, stderr_destination};
use rustc_errors::json::JsonEmitter;
use rustc_interface::interface::Config;
use rustc_session::config::ErrorOutputType;

use crate::emitter::cgp_emitter::CgpEmitter;

/// Install the transforming emitter on the compiler session, replicating how rustc builds its
/// default emitter so cargo's output is what vanilla `rustc` would produce apart from the CGP
/// transforms.
///
/// It handles both the JSON error format — the one the front-end drives cargo with — and the
/// human-readable one a direct `cargo-cgp-driver` (or `cargo cgp check`) invocation renders,
/// rebuilding whichever emitter the compiler's `default_emitter` would for that format and
/// wrapping it in [`CgpEmitter`]. The session options the emitter construction needs are not
/// reachable from `psess_created`'s `&mut ParseSess`, so they are read here from
/// [`Config::opts`] and moved into the callback.
pub fn install(config: &mut Config) {
    let sopts = &config.opts;

    let macro_backtrace = sopts.unstable_opts.macro_backtrace;
    let track_diagnostics = sopts.unstable_opts.track_diagnostics;
    let ui_testing = sopts.unstable_opts.ui_testing;
    let link_only = sopts.unstable_opts.link_only;
    let diagnostic_width = sopts.diagnostic_width;
    let ignored_directories = sopts
        .unstable_opts
        .ignore_directory_in_diagnostics_source_blocks
        .clone();
    let terminal_url = resolve_terminal_url(
        sopts.unstable_opts.terminal_urls,
        sopts.unstable_features.is_nightly_build(),
    );
    // Copy the format out (it is `Copy`) so the immutable borrow of `config.opts` ends
    // before `config.psess_created` is assigned.
    let error_format = sopts.error_format;

    match error_format {
        ErrorOutputType::Json {
            pretty,
            json_rendered,
            color_config,
        } => {
            config.psess_created = Some(Box::new(move |psess| {
                let source_map = (!link_only).then(|| psess.clone_source_map());
                let inner = JsonEmitter::new(
                    Box::new(io::BufWriter::new(io::stderr())),
                    source_map,
                    pretty,
                    json_rendered,
                    color_config,
                )
                .ui_testing(ui_testing)
                .ignored_directories_in_source_blocks(ignored_directories)
                .diagnostic_width(diagnostic_width)
                .macro_backtrace(macro_backtrace)
                .track_diagnostics(track_diagnostics)
                .terminal_url(terminal_url);

                psess.dcx().set_emitter(Box::new(CgpEmitter::new(inner)));
            }));
        }
        ErrorOutputType::HumanReadable {
            kind: HumanReadableErrorType { short, unicode },
            color_config,
        } => {
            config.psess_created = Some(Box::new(move |psess| {
                let source_map = (!link_only).then(|| psess.clone_source_map());
                let inner = AnnotateSnippetEmitter::new(stderr_destination(color_config))
                    .sm(source_map)
                    .short_message(short)
                    .diagnostic_width(diagnostic_width)
                    .macro_backtrace(macro_backtrace)
                    .track_diagnostics(track_diagnostics)
                    .terminal_url(terminal_url)
                    .theme(if unicode {
                        OutputTheme::Unicode
                    } else {
                        OutputTheme::Ascii
                    })
                    .ignored_directories_in_source_blocks(ignored_directories)
                    .ui_testing(ui_testing);

                psess.dcx().set_emitter(Box::new(CgpEmitter::new(inner)));
            }));
        }
    }
}

/// Resolve `--terminal-urls=auto` the same way `rustc_session::session::default_emitter`
/// does, so the rebuilt emitter matches the compiler's default rather than guessing.
fn resolve_terminal_url(setting: TerminalUrl, is_nightly: bool) -> TerminalUrl {
    match setting {
        TerminalUrl::Auto => {
            match (
                std::env::var("COLORTERM").as_deref(),
                std::env::var("TERM").as_deref(),
            ) {
                (Ok("truecolor"), Ok("xterm-256color")) if is_nightly => TerminalUrl::Yes,
                _ => TerminalUrl::No,
            }
        }
        other => other,
    }
}
