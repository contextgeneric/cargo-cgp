//! Resugaring `Symbol!` — the type-level string every CGP field name travels as.
//!
//! `Symbol!("height")` expands to a byte length and a right-folded character list closed by
//! `Nil`, so this pass reads that spine back into the literal the programmer wrote. The match
//! is exact: the declared length must equal the decoded string's byte length, the spine must be
//! `Chars` all the way down to `Nil`, and every head must be a plain character literal.

use quote::ToTokens;
use syn::visit_mut::{self, VisitMut};
use syn::{GenericArgument, LitStr, Type};

use crate::resugar::parts::{Delimiter, is_terminator, macro_type, type_args};

/// The `Symbol!` pass. Runs before the [path](super::path) and [list](super::list) passes,
/// which read the `Symbol!("…")` calls it produces.
pub struct Symbols;

impl VisitMut for Symbols {
    fn visit_type_mut(&mut self, ty: &mut Type) {
        visit_mut::visit_type_mut(self, ty);

        if let Some(name) = symbol_spine_name(ty) {
            *ty = symbol_macro(&name);
        }
    }
}

/// Build the `Symbol!("name")` call a decoded spine becomes.
pub(crate) fn symbol_macro(name: &str) -> Type {
    let literal = LitStr::new(name, proc_macro2::Span::call_site());
    macro_type("Symbol", Delimiter::Paren, literal.to_token_stream())
}

/// The string inside a resugared `Symbol!("name")` call, or `None` when `ty` is not one.
///
/// The later passes read a symbol back this way rather than re-decoding a spine, since by the
/// time they run this pass has already folded every well-formed spine into a call.
pub(crate) fn symbol_macro_name(ty: &Type) -> Option<String> {
    let Type::Macro(mac) = ty else {
        return None;
    };
    if mac.mac.path.segments.last()?.ident != "Symbol" {
        return None;
    }
    let literal: LitStr = syn::parse2(mac.mac.tokens.clone()).ok()?;
    Some(literal.value())
}

/// Decode a `Symbol<LEN, Chars<'a', … Nil>>` spine to the string it encodes, or `None` on any
/// mismatch. The length is compared against the decoded string's **byte** length, because that
/// is what `Symbol!` bakes in (`str::len()`, not the character count).
fn symbol_spine_name(ty: &Type) -> Option<String> {
    let args = type_args(ty, "Symbol")?;
    let [length, spine] = args.as_slice() else {
        return None;
    };

    let GenericArgument::Const(length) = length else {
        return None;
    };
    let syn::Expr::Lit(literal) = length else {
        return None;
    };
    let syn::Lit::Int(declared) = &literal.lit else {
        return None;
    };
    let declared: usize = declared.base10_parse().ok()?;

    let GenericArgument::Type(spine) = spine else {
        return None;
    };
    let name = chars_spine(spine)?;

    (declared == name.len()).then_some(name)
}

/// Decode a `Chars<'c', tail>` chain terminated by `Nil` into the string it spells.
fn chars_spine(ty: &Type) -> Option<String> {
    if is_terminator(ty, "Nil") {
        return Some(String::new());
    }

    let args = type_args(ty, "Chars")?;
    let [head, tail] = args.as_slice() else {
        return None;
    };

    let GenericArgument::Const(head) = head else {
        return None;
    };
    let syn::Expr::Lit(literal) = head else {
        return None;
    };
    let syn::Lit::Char(character) = &literal.lit else {
        return None;
    };

    let GenericArgument::Type(tail) = tail else {
        return None;
    };

    let mut name = String::new();
    name.push(character.value());
    name.push_str(&chars_spine(tail)?);
    Some(name)
}
