//! The naming rule CGP macros follow for the names they invent.

/// Whether `name` is a name CGP's macros reserve for an item they generate: two underscores at each
/// end of a non-empty name that neither starts nor ends with an underscore of its own
/// (`__Context__`, `__a__`), the convention every CGP macro follows, so a name with a lone or an
/// inner pair of underscores (`__init`, `snake__case`) is the program's own.
pub fn is_reserved_name(name: &str) -> bool {
    name.strip_prefix("__")
        .and_then(|tail| tail.strip_suffix("__"))
        .is_some_and(|inner| !inner.is_empty() && !inner.starts_with('_') && !inner.ends_with('_'))
}

/// The reserved identifiers in `text`: names wrapped in double underscores, the convention every
/// CGP macro follows for the names it invents (`__Context__`, and the lifetimes `'__a__` and
/// `'__a0__` `#[cgp_auto_dispatch]` generates), each returned without a lifetime's quote. A
/// suggestion that inserts one names something the program cannot write, since only the
/// expansion declares it.
pub fn reserved_names(text: &str) -> Vec<&str> {
    let is_ident = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let mut names = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("__") {
        let preceded = rest[..start].chars().next_back().is_some_and(is_ident);
        let word_end = rest[start..]
            .find(|c: char| !is_ident(c))
            .map_or(rest.len(), |len| start + len);
        let word = &rest[start..word_end];
        if !preceded && is_reserved_name(word) {
            names.push(word);
        }
        rest = &rest[word_end.max(start + 2)..];
    }
    names
}
