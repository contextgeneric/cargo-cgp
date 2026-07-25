//! Printing the expanded crate, resugared.

use std::fs;

use cargo_cgp_expand::{ExpandOptions, ItemPath, resugar_expanded_source};
use rustc_ast_pretty::pprust;
use rustc_middle::ty::TyCtxt;
use rustc_session::Session;

use crate::expand::request::ExpandRequest;

/// A no-op annotator: the expansion is printed plain, with none of the node ids or hygiene marks
/// the compiler's other pretty-printing modes interleave. rustc's own equivalent is private, so
/// this is the one-line stand-in.
struct NoAnn;

impl pprust::PpAnn for NoAnn {}

/// Print the expanded crate, resugar its CGP constructs, and write the result where the front-end
/// asked for it.
///
/// This runs from the `after_expansion` callback, which is the earliest point the expanded AST
/// exists — and the reason the driver does not simply set `-Zunpretty=expanded`: under that flag
/// the compiler prints the crate and exits *before* any callback runs, so the driver would never
/// get to resugar anything.
///
/// The crate is printed through the compiler's own [`pprust::print_crate`], the same call
/// `-Zunpretty=expanded` makes, so the text handed to the resugaring is exactly what `cargo-expand`
/// would show. `is_expanded` is set for the same reason it is there: it prints the faked
/// `#![feature(prelude_import)]` / `#![no_std]` preamble that stops the printed source from
/// re-injecting libstd.
pub fn print_expansion(sess: &Session, tcx: TyCtxt<'_>, request: &ExpandRequest) {
    let source_name = sess.io.input.file_name(sess);
    let Some(source) = source_text(sess, &source_name) else {
        // Without the crate root's text the printer cannot interleave comments; that only happens
        // for an input the source map never loaded, where there is nothing to expand either.
        return;
    };

    // Expansion has already run — the compiler forces it immediately before calling us — so the
    // expanded crate is sitting behind this query's `Steal`, unstolen until lowering.
    let printed = tcx.resolver_for_lowering().1.borrow().clone();
    let printed = pprust::print_crate(
        sess.source_map(),
        &printed,
        source_name,
        source,
        &NoAnn,
        true,
        sess.psess.edition,
        &sess.psess.attr_id_generator,
    );

    let Some(options) = expand_options(sess, request) else {
        return;
    };
    let resugared = resugar_expanded_source(&printed, &options);

    // An item filter that selected nothing is reported *here*, by the layer that knows: the
    // front-end sees only an absent expansion, and cannot tell a path that names nothing from a
    // crate that never got far enough to expand. Guessing between them is what made this confusing.
    //
    // Printed straight to stderr rather than emitted as a compiler diagnostic, for two reasons. It is
    // not about the code being compiled, so it should not add to the crate's error count and make
    // cargo report a failed compilation; and the driver's own diagnostic post-processing would rewrite
    // it — the module-path strip exists to shorten CGP type names in an error, and it would shorten
    // the very path this message quotes (`contexts::nope` down to `nope`).
    if resugared.is_empty()
        && let Some(item) = &request.item
    {
        eprintln!("error: no module or item matched `{item}`");
        eprintln!(
            "note: the path names a module or item inside the crate being expanded, as in \
             `contexts::app`; a leading `crate::` is accepted too"
        );
        return;
    }

    if let Err(error) = fs::write(&request.output, resugared) {
        sess.dcx()
            .warn(format!("could not write the expansion: {error}"));
    }
}

/// Build the resugaring options for this request, or `None` when its item path is unusable.
///
/// The front-end checks the path's shape before compiling, so a rejected one here means the two
/// disagree; warning rather than expanding the whole crate keeps that from looking like a filter that
/// silently did nothing.
fn expand_options(sess: &Session, request: &ExpandRequest) -> Option<ExpandOptions> {
    let item = match &request.item {
        Some(item) => match ItemPath::parse(item) {
            Some(path) => Some(path),
            None => {
                sess.dcx().warn(format!(
                    "`{item}` is not an item path, so nothing was expanded"
                ));
                return None;
            }
        },
        None => None,
    };

    Some(ExpandOptions {
        item,
        ..ExpandOptions::default()
    })
}

/// The crate root's source text, read back from the source map the way the compiler's own
/// pretty-printing does.
fn source_text(sess: &Session, source_name: &rustc_span::FileName) -> Option<String> {
    let file = sess.source_map().get_source_file(source_name)?;
    file.src.as_ref().map(|src| String::clone(src))
}
