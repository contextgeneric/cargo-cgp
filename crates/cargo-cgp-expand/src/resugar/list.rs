//! Resugaring `Product!` and `Sum!` — the type-level list spines.
//!
//! A product expands through `Cons` to `Nil` and a sum through `Either` to `Void`, so this pass
//! collects each spine's heads and prints them as the flat list the programmer wrote.
//!
//! Unlike the diagnostic resugarers, it stops there: a list whose elements are all named fields is
//! **not** folded on to the `Struct! { … }` / `Enum! { … }` record form. Those forms are
//! presentation-only — no such CGP macros exist — and this pass writes *source*, where every
//! construct shown should be syntax the programmer could have written. A reader of an expansion
//! sees `Product![Field<Symbol!("width"), f64>, …]`, which is both real and true to the type.

use proc_macro2::TokenStream;
use quote::quote;
use syn::visit_mut::{self, VisitMut};
use syn::{GenericArgument, Type};

use crate::resugar::parts::{Delimiter, is_terminator, macro_type, type_args};

/// The `Product!`/`Sum!` pass. Runs after [`Paths`](super::Paths), which consumes the `Nil` that
/// terminates a path — a `Nil` this pass would otherwise read as an empty list.
pub struct Lists;

impl VisitMut for Lists {
    fn visit_type_mut(&mut self, ty: &mut Type) {
        // **Outermost-first.** A spine is right-nested, so its tail is a spine too: folding the
        // innermost cell first — which is what an ordinary visitor would do — replaces that tail
        // with a macro call, and the enclosing cells then no longer match, leaving a two-element
        // list as `Cons<A, Product![B]>`. So a cell is folded before the recursion, and the
        // recursion runs over its collected elements instead.
        if let Some(folded) = self.fold_spine(ty) {
            *ty = folded;
            return;
        }

        visit_mut::visit_type_mut(self, ty);
    }
}

impl Lists {
    /// Fold `ty` if it is a product or sum spine, resugaring inside each element first so a nested
    /// list — or a field's list-typed value — folds in turn.
    fn fold_spine(&mut self, ty: &Type) -> Option<Type> {
        if let Some(elements) = self.spine(ty, "Cons", "Nil") {
            return Some(list_macro("Product", &elements));
        }
        if let Some(elements) = self.spine(ty, "Either", "Void") {
            return Some(list_macro("Sum", &elements));
        }
        None
    }

    /// The elements of a spine, each already resugared.
    fn spine(&mut self, ty: &Type, cell: &str, terminator: &str) -> Option<Vec<Type>> {
        let mut elements = spine_elements(ty, cell, terminator)?;
        for element in &mut elements {
            self.visit_type_mut(element);
        }
        Some(elements)
    }
}

/// Build the `Product![A, B]` / `Sum![A, B]` call a spine becomes.
fn list_macro(name: &str, elements: &[Type]) -> Type {
    let body: TokenStream = quote!(#(#elements),*);
    macro_type(name, Delimiter::Bracket, body)
}

/// Collect the head types of a `Cell<Head, Tail>` spine ended by `Terminator`, or `None` when
/// `ty` is not such a spine.
///
/// The first node must be a cell, so a bare terminator is left as the plain type it reads as
/// rather than resugared into an empty list, and a tail that is neither a further cell nor the
/// terminator declines, so only a closed list is folded.
fn spine_elements(ty: &Type, cell: &str, terminator: &str) -> Option<Vec<Type>> {
    let mut elements = Vec::new();
    let mut current = ty.clone();

    loop {
        let args = type_args(&current, cell)?;
        let [GenericArgument::Type(head), GenericArgument::Type(tail)] = args.as_slice() else {
            return None;
        };

        elements.push(head.clone());

        if is_terminator(tail, terminator) {
            return Some(elements);
        }
        current = tail.clone();
    }
}
