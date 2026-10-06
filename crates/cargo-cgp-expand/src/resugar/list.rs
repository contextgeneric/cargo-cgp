//! Resugaring `Product!` and `Sum!`, the type-level list spines, and their `Struct!`/`Enum!` shapes.
//!
//! A product expands through `Cons` to `Nil` and a sum through `Either` to `Void`, so this pass
//! collects each spine's heads and prints them as the flat list the programmer wrote. A list whose
//! elements are all `Field` cells folds one step further, to the CGP `Struct!` or `Enum!` shape it
//! describes, when the shape has an exact spelling: `Struct! { width: f64 }`, `Struct!(u8, u16)`,
//! or `Enum! { Empty, Circle(f64) }`. Any other list keeps its `Product!`/`Sum!` form. The rules
//! are the same as the diagnostic resugarers' (see
//! `cgp-knowledge-base/cargo-cgp/implementation/resugaring.md`).

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::visit_mut::{self, VisitMut};
use syn::{Expr, ExprLit, GenericArgument, Lit, MacroDelimiter, Type};

use crate::resugar::parts::{Delimiter, is_terminator, macro_type, shape_name, type_args};
use crate::resugar::symbol::symbol_macro_name;

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
            return Some(
                struct_shape(&elements).unwrap_or_else(|| list_macro("Product", &elements)),
            );
        }
        if let Some(elements) = self.spine(ty, "Either", "Void") {
            return Some(enum_shape(&elements).unwrap_or_else(|| list_macro("Sum", &elements)));
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

/// The tag of a `Field` cell: a field or variant name, or a tuple position.
enum Tag {
    Name(Ident),
    Index(usize),
}

/// The `Struct!` a product of `Field` cells becomes, or `None` when it has no exact spelling.
///
/// Every tag a writable name gives `Struct! { a: A }`. Every tag an `Index`, counting up from 0 with
/// at least two cells, gives `Struct!(A, B)`. A single `Index` cell has no spelling, because
/// `Struct!(T)` is the bare `T`, and neither has a list that mixes the tags or skips a position.
fn struct_shape(elements: &[Type]) -> Option<Type> {
    let cells = field_cells(elements)?;

    let named = cells
        .iter()
        .map(|(tag, value)| match tag {
            Tag::Name(name) => Some(quote!(#name: #value)),
            Tag::Index(_) => None,
        })
        .collect::<Option<Vec<_>>>();
    if let Some(fields) = named {
        return Some(macro_type("Struct", Delimiter::Brace, quote!(#(#fields),*)));
    }

    if cells.len() < 2 {
        return None;
    }
    let positional = cells
        .iter()
        .enumerate()
        .map(|(position, (tag, value))| match tag {
            Tag::Index(index) if *index == position => Some(value),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    Some(macro_type(
        "Struct",
        Delimiter::Paren,
        quote!(#(#positional),*),
    ))
}

/// The `Enum!` a sum of named `Field` cells becomes, or `None` when it has no exact spelling.
///
/// Each variant takes the shortest of its equivalent spellings: a `Nil` payload is a unit variant,
/// a payload already folded to `Struct! { … }` or `Struct!(…)` lends the variant its braces or
/// parentheses, and any other payload is the single positional field of `V(T)`.
fn enum_shape(elements: &[Type]) -> Option<Type> {
    let variants = field_cells(elements)?
        .into_iter()
        .map(|(tag, payload)| {
            let Tag::Name(name) = tag else {
                return None;
            };
            Some(variant(&name, &payload))
        })
        .collect::<Option<Vec<_>>>()?;

    Some(macro_type("Enum", Delimiter::Brace, quote!(#(#variants),*)))
}

/// One `Enum!` variant, spelled from its payload.
fn variant(name: &Ident, payload: &Type) -> TokenStream {
    if is_terminator(payload, "Nil") {
        return quote!(#name);
    }

    if let Type::Macro(shape) = payload
        && shape.mac.path.is_ident("Struct")
    {
        let body = &shape.mac.tokens;
        match shape.mac.delimiter {
            MacroDelimiter::Brace(_) => return quote!(#name { #body }),
            MacroDelimiter::Paren(_) => return quote!(#name(#body)),
            MacroDelimiter::Bracket(_) => {}
        }
    }

    quote!(#name(#payload))
}

/// Interpret every element as a `Field<Tag, Value>` cell whose tag is a writable `Symbol!` name or
/// an `Index<N>` position, or `None` if any element is not such a cell.
fn field_cells(elements: &[Type]) -> Option<Vec<(Tag, Type)>> {
    elements
        .iter()
        .map(|element| {
            let args = type_args(element, "Field")?;
            let [GenericArgument::Type(tag), GenericArgument::Type(value)] = args.as_slice() else {
                return None;
            };
            let tag = if let Some(name) = symbol_macro_name(tag) {
                Tag::Name(syn::parse_str(&shape_name(&name)?).ok()?)
            } else {
                Tag::Index(index_tag(tag)?)
            };
            Some((tag, value.clone()))
        })
        .collect()
}

/// The `N` of an `Index<N>` tag written as a plain decimal literal.
fn index_tag(tag: &Type) -> Option<usize> {
    let args = type_args(tag, "Index")?;
    let [
        GenericArgument::Const(Expr::Lit(ExprLit {
            lit: Lit::Int(index),
            ..
        })),
    ] = args.as_slice()
    else {
        return None;
    };
    if !index.suffix().is_empty() {
        return None;
    }
    index.base10_parse().ok()
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
