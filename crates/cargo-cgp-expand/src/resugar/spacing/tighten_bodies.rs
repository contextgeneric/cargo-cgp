//! Tightening the spacing the printer leaves inside a resugared macro call, by removing whitespace
//! alone; see `cgp-knowledge-base/cargo-cgp/implementation/expand-command.md`.

use crate::resugar::spacing::layout::{Layout, join_lines, tighten};
use crate::resugar::spacing::scan::{body_end, next_sugar};

/// Remove the spaces the printer inserted inside every resugared macro body in `printed`.
pub fn tighten_sugar_bodies(printed: &str) -> String {
    let mut out = String::with_capacity(printed.len());
    let mut rest = printed;

    while let Some((index, open, close, layout)) = next_sugar(rest) {
        let (before, from_macro) = rest.split_at(index);
        out.push_str(before);

        // Copy the name and opening delimiter through, then tighten the balanced body.
        let name_end = from_macro
            .find(open)
            .expect("the delimiter was just matched")
            + 1;
        out.push_str(&from_macro[..name_end]);
        let body_start = &from_macro[name_end..];

        match (body_end(body_start, open, close), layout) {
            (Some(end), Layout::AsPrinted) => {
                out.push_str(&tighten(&body_start[..end]));
                rest = &body_start[end..];
            }
            (Some(end), Layout::Inline) => {
                // `end` is the closing delimiter, which `rest` carries on.
                out.push(' ');
                out.push_str(&tighten(&join_lines(&body_start[..end])));
                out.push(' ');
                rest = &body_start[end..];
            }
            // An unbalanced body cannot be trusted, so it is copied through untouched.
            (None, _) => rest = body_start,
        }
    }

    out.push_str(rest);
    out
}
