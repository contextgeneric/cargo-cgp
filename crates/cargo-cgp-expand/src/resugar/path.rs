//! Resugaring `Path!`: the type-level path a namespace or an `open` statement dispatches on.
//!
//! `Path!(@app.GreeterComponent)` expands to a right-nested `PathCons` chain closed by `Nil`,
//! and this pass reads it back. Segments are classified the way `Path!` classifies them going
//! forward: a lowercase, non-primitive identifier is a `Symbol`, and every other segment was
//! kept verbatim as a named type. Two shapes decline, leaving the raw spine rather than risking
//! a mangled path: a module-qualified segment that is not a plain type path, and a bare lowercase
//! identifier, which `Path!` would have encoded as a `Symbol`.
//!
//! Only a `Nil`-terminated chain is folded. An **open-ended** path, whose tail is a generic "rest
//! of path" parameter, has no `Path!` spelling, since `Path!(@a.b)` is the closed path, and this
//! pass writes source, so such a chain is left as it stands. In an expansion the open tail is a
//! named generic parameter, not the `_` a diagnostic shows.

use cargo_cgp_error_processing::{is_ident, is_primitive_type};
use proc_macro2::{Punct, Spacing, TokenStream, TokenTree};
use quote::ToTokens;
use syn::visit_mut::{self, VisitMut};
use syn::{GenericArgument, Type};

use crate::resugar::parts::{Delimiter, is_terminator, macro_type, type_args};
use crate::resugar::symbol::symbol_macro_name;

/// The `Path!` pass. Runs after [`Symbols`](super::Symbols), whose `Symbol!("…")` calls it reads
/// as path segments, and before the [list](super::list) pass, so a path's terminating `Nil` is
/// consumed here rather than mistaken for an empty `Product!`.
pub struct Paths;

impl VisitMut for Paths {
    fn visit_type_mut(&mut self, ty: &mut Type) {
        // A `PathCons` chain is folded whole or left whole. Outermost-first, for the same reason
        // the [list](super::list) pass is: the chain is right-nested, so folding an inner link
        // first leaves the outer links unmatched, and without recursing when the fold declines,
        // since folding a *part* of a declining chain would print a `Path!` nested inside a
        // `PathCons`, which is no path at all.
        if type_args(ty, "PathCons").is_some() {
            if let Some(segments) = self.path_spine(ty) {
                *ty = path_macro(&segments);
            }
            return;
        }

        visit_mut::visit_type_mut(self, ty);
    }
}

/// Build the `Path!(@a.b.C)` call a decoded spine becomes.
///
/// The macro form is used rather than the bare `@…` a diagnostic shows, because this output is
/// source code, where a path type is written as the macro call. The `@` and `.` separators are
/// emitted with joint spacing so the printer lays the path out as one word (`@app.Greeter`) rather
/// than spacing every token apart.
fn path_macro(segments: &[TokenStream]) -> Type {
    let mut body = TokenStream::from(TokenTree::Punct(Punct::new('@', Spacing::Joint)));
    for (index, segment) in segments.iter().enumerate() {
        if index > 0 {
            body.extend([TokenTree::Punct(Punct::new('.', Spacing::Joint))]);
        }
        body.extend(segment.clone());
    }

    macro_type("Path", Delimiter::Paren, body)
}

impl Paths {
    /// Walk a `PathCons<Head, Tail>` chain into its rendered segments. `None` unless the chain is
    /// `PathCons` all the way down to `Nil` and every segment round-trips through `Path!`.
    fn path_spine(&mut self, ty: &Type) -> Option<Vec<TokenStream>> {
        let mut segments = Vec::new();
        let mut current = ty.clone();

        loop {
            let args = type_args(&current, "PathCons")?;
            let [GenericArgument::Type(head), GenericArgument::Type(tail)] = args.as_slice() else {
                return None;
            };

            // Resugar inside the segment first, so a construct nested in a compound value type
            // (`Vec<Product![…]>`) is folded before the segment is rendered to tokens.
            let mut head = head.clone();
            visit_mut::visit_type_mut(self, &mut head);
            segments.push(render_segment(&head)?);

            if is_terminator(tail, "Nil") {
                return Some(segments);
            }
            current = tail.clone();
        }
    }
}

/// Render one spine segment back to the tokens `Path!` was written with, or `None` when it would
/// not round-trip.
fn render_segment(ty: &Type) -> Option<TokenStream> {
    // A `Symbol!("app")` head is a lowercase path segment, but only when the name really is one
    // `Path!` would have encoded as a symbol.
    if let Some(name) = symbol_macro_name(ty) {
        let is_lowercase_ident = is_ident(&name)
            && name.starts_with(|c: char| c.is_ascii_lowercase())
            && !is_primitive_type(&name);
        if !is_lowercase_ident {
            return None;
        }
        let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
        return Some(ident.to_token_stream());
    }

    if let Type::Path(path) = ty
        && path.qself.is_none()
    {
        let plain_idents = path
            .path
            .segments
            .iter()
            .all(|segment| segment.arguments.is_none());
        let qualified = path.path.segments.len() > 1;
        // A qualified segment must be a plain type path, as the diagnostic pass requires.
        if qualified && !plain_idents {
            return None;
        }
        if plain_idents {
            let tail = path.path.segments.last()?.ident.to_string();
            // A bare lowercase identifier would have been a `Symbol`, so meeting one as a plain
            // type is ambiguous rather than a clean path, and so is a qualified path ending in one.
            if tail.starts_with(|c: char| c.is_ascii_lowercase()) && !is_primitive_type(&tail) {
                return None;
            }
        }
    }

    // Everything else is kept verbatim, as `Path!` kept it going forward: a named type, whose
    // qualifier says which type it is, or a compound value type an `open` statement dispatches on
    // (`Vec<u8>`, `&Coord`, `DateTime<Utc>`).
    Some(ty.to_token_stream())
}
