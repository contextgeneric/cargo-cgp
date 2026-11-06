//! How Rust and CGP's `Path!` spell a name, for the text resugaring and the expansion's alike.

/// Whether `s` is a single Rust identifier (ASCII letters, digits, and underscores, not
/// starting with a digit).
pub fn is_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c == '_' || c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

/// Whether `ident` is a Rust primitive type name, matching the `Path!` macro's own rule
/// (`cgp-macro-core`'s `path_element::is_primitive_type`): only the real names, so `f` or `u2`
/// is a symbol.
pub fn is_primitive_type(ident: &str) -> bool {
    matches!(
        ident,
        "u8" | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "f32"
            | "f64"
            | "char"
            | "bool"
            | "str"
    )
}

/// A trait or type `name` with its generic `arguments` in angle brackets, as the reader writes it,
/// or the bare `name` when there are none: `AreaCalculator` with the argument `Shape` is
/// `AreaCalculator<Shape>`.
pub fn with_arguments(name: &str, arguments: &[String]) -> String {
    if arguments.is_empty() {
        name.to_owned()
    } else {
        format!("{name}<{}>", arguments.join(", "))
    }
}
