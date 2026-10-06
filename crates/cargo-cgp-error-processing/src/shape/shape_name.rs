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

/// A tag's name as a `Struct!` or `Enum!` body writes it, or `None` when no body can write it.
///
/// A `Symbol!` tag holds any string, but a body can only name a field or variant with an
/// identifier, so a name such as `"foo bar"` has no record spelling. A keyword is written raw
/// (`type` → `r#type`), which `Struct!` tags by its logical name, so the round trip is exact.
pub fn shape_name(name: &str) -> Option<String> {
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
