//! Shared pieces the resugaring passes are built from: reading a type's shape, and building
//! the macro-call type a resugared construct becomes.

use proc_macro2::TokenStream;
use syn::token::{Brace, Bracket, Paren};
use syn::{GenericArgument, Macro, MacroDelimiter, PathArguments, Type, TypeMacro};

/// Which delimiter a resugared macro call uses: `Symbol!(…)`, `Path!(…)`, and a tuple
/// `Struct!(…)` take parentheses, `Product![…]` and `Sum![…]` brackets, and a named
/// `Struct! { … }` and `Enum! { … }` braces.
#[derive(Clone, Copy)]
pub(crate) enum Delimiter {
    Paren,
    Bracket,
    Brace,
}

/// Build the macro-call type a resugared construct becomes, such as `Symbol!("height")`.
///
/// The output has to be a syntax node rather than a string, because the printer formats the tree
/// after every pass has run, which lets a resugared call be laid out compactly where the raw spine
/// it replaced was broken across lines.
pub(crate) fn macro_type(name: &str, delimiter: Delimiter, tokens: TokenStream) -> Type {
    let delimiter = match delimiter {
        Delimiter::Paren => MacroDelimiter::Paren(Paren::default()),
        Delimiter::Bracket => MacroDelimiter::Bracket(Bracket::default()),
        Delimiter::Brace => MacroDelimiter::Brace(Brace::default()),
    };

    Type::Macro(TypeMacro {
        mac: Macro {
            path: syn::parse_str(name).expect("macro name is a valid path"),
            bang_token: Default::default(),
            delimiter,
            tokens,
        },
    })
}

/// The generic arguments of `ty` when it is a path type whose **last** segment is `name`.
///
/// Only the last segment is compared, so a construct the macros emit fully qualified
/// (`::cgp::macro_prelude::Symbol<…>`) matches whether or not the qualifier was stripped. A
/// path with no arguments yields an empty list, which is how a spine terminator is recognized.
pub(crate) fn type_args(ty: &Type, name: &str) -> Option<Vec<GenericArgument>> {
    let Type::Path(path) = ty else {
        return None;
    };
    let segment = path.path.segments.last()?;
    if segment.ident != name {
        return None;
    }
    match &segment.arguments {
        PathArguments::AngleBracketed(args) => Some(args.args.iter().cloned().collect()),
        PathArguments::None => Some(Vec::new()),
        PathArguments::Parenthesized(_) => None,
    }
}

/// Whether `ty` is the bare path `name` carrying no arguments, a spine terminator such as
/// `Nil` or `Void`.
pub(crate) fn is_terminator(ty: &Type, name: &str) -> bool {
    type_args(ty, name).is_some_and(|args| args.is_empty())
}
