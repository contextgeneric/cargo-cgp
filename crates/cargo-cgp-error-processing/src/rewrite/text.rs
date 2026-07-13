//! Small text-splitting utilities shared by the wiring-message parsers and rewrites.

/// The last `::`-separated segment of a path, so `cgp::prelude::IsProviderFor` becomes
/// `IsProviderFor` and a component key matches the compiler's unqualified item name.
pub(crate) fn last_segment(path: &str) -> &str {
    path.rsplit("::").next().unwrap_or(path).trim()
}

/// Split `Path<args>` into its path (`Path`) and the raw argument text (`args`), requiring
/// the string to be a single generic application closed by a trailing `>`.
pub(crate) fn split_generics(s: &str) -> Option<(&str, &str)> {
    let lt = s.find('<')?;
    let args = s.strip_suffix('>')?.get(lt + 1..)?;
    Some((&s[..lt], args))
}

/// Split a generic argument list on its top-level commas, so a nested `<…>`, `(…)`, or
/// `[…]` argument (e.g. a generic context or a `Params` tuple) is kept whole.
pub(crate) fn split_top_level(args: &str) -> Vec<&str> {
    let mut depth: i32 = 0;
    let mut parts = Vec::new();
    let mut start = 0;
    for (i, c) in args.char_indices() {
        match c {
            '<' | '(' | '[' => depth += 1,
            '>' | ')' | ']' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(&args[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&args[start..]);
    parts
}
