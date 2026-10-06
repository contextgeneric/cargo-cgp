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
/// The output has to be a syntax node rather than a string, because the printer formats the
/// tree after every pass has run — which is what lets a resugared call be laid out compactly
/// where the raw spine it replaced was broken across lines.
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

/// Whether `ty` is the bare path `name` carrying no arguments — a spine terminator such as
/// `Nil` or `Void`.
pub(crate) fn is_terminator(ty: &Type, name: &str) -> bool {
    type_args(ty, name).is_some_and(|args| args.is_empty())
}

/// Whether `s` is a single Rust identifier.
pub(crate) fn is_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c == '_' || c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

/// Whether `ident` names a primitive type, matching the `Path!` macro's own rule
/// (`cgp-macro-core`'s `path_element::is_primitive_type`): an `i`/`u`/`f` followed by digits,
/// or one of `char`/`bool`/`usize`/`isize`/`str`. Kept in step with the same rule in the
/// diagnostic resugarers, so a path segment classifies identically wherever it is met.
pub(crate) fn is_primitive_type(ident: &str) -> bool {
    if (ident.starts_with('i') || ident.starts_with('u') || ident.starts_with('f'))
        && ident.len() > 1
        && ident[1..].chars().all(|c| c.is_ascii_digit())
    {
        return true;
    }
    matches!(ident, "char" | "bool" | "usize" | "isize" | "str")
}

/// The keywords a field or variant name can only be written as with `r#`: the strict keywords and
/// the reserved ones, as of the 2024 edition.
const RAW_KEYWORDS: &[&str] = &[
    "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "do", "dyn",
    "else", "enum", "extern", "false", "final", "fn", "for", "gen", "if", "impl", "in", "let",
    "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref", "return",
    "static", "struct", "trait", "true", "try", "type", "typeof", "unsafe", "unsized", "use",
    "virtual", "where", "while", "yield",
];

/// The keywords that cannot be a field or variant name at all, not even as a raw identifier.
const UNWRITABLE_KEYWORDS: &[&str] = &["_", "crate", "self", "Self", "super"];

/// A `Symbol!` tag's name as a `Struct!` or `Enum!` body writes it, or `None` when no body can
/// write it: an identifier as is, a keyword raw (`r#type`), and anything else not at all. Kept in
/// step with `shape_name` in the diagnostic resugarers (`cargo-cgp-error-processing`), so a shape
/// gets the same spelling in an expansion and in a diagnostic.
pub(crate) fn shape_name(name: &str) -> Option<String> {
    let mut chars = name.chars();
    let first = chars.next()?;

    if !(first == '_' || first.is_alphabetic()) || !chars.all(|c| c == '_' || c.is_alphanumeric()) {
        return None;
    }

    if UNWRITABLE_KEYWORDS.contains(&name) {
        return None;
    }

    if RAW_KEYWORDS.contains(&name) {
        return Some(format!("r#{name}"));
    }

    Some(name.to_owned())
}
