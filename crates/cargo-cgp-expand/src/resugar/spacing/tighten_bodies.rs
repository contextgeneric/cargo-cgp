//! Tightening the spacing the printer leaves inside a resugared macro call.
//!
//! A resugared construct is a macro call whose body holds ordinary types, and the printer lays a
//! macro body out token by token: it has no way to know the body is a type list, so it spaces
//! every token apart and prints `Product![Multiply < Symbol!("foo") >]`. Its spacing rules cannot
//! be coaxed into the conventional form — the space *before* a token is the printer's decision and
//! an identifier cannot ask for it to be dropped — so the layout is corrected after printing.
//!
//! The printer also always breaks a brace-delimited macro body onto lines of its own, printing a
//! `Struct! { … }` shape as a block. It still puts the whole body on one line, so a brace body is
//! joined back onto the macro's line, `Struct! { width: f64 }`, the way a diagnostic and the
//! documentation write a shape.
//!
//! This is the one text pass in the crate, and it is deliberately narrow: it only ever removes
//! whitespace (joining a brace body replaces each line break with a single space), only inside the
//! body of a call to one of the CGP macros `SUGAR_MACROS` names, and never inside a literal. The
//! search for those calls steps over literals and comments, so text that merely names a macro, such
//! as `"Sum![a , b]"` in a string or a doc comment, is left alone. So it cannot alter the meaning of
//! anything, and it cannot reach the surrounding program at all. (A
//! `Product![…]` a programmer wrote themselves is tightened the same way, which is the same
//! transformation and equally harmless.)

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
