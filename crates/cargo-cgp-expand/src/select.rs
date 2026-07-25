//! Selecting one module or item out of an expansion.
//!
//! An expanded crate is large, and a reader usually wants one part of it: the module they are
//! working in, or the type whose generated impls they are checking. `cargo cgp expand --item <path>`
//! narrows the output to that path, and this module is the narrowing.
//!
//! Three rules decide what a path selects, and the third is the CGP-shaped one:
//!
//! 1. An item **declared** at that path — a struct, trait, function, and so on. A module selects its
//!    *contents* rather than the `mod` wrapper, since the wrapper is noise around what was asked for.
//! 2. An `impl` block whose **self type** is that path, so `--item Rectangle` shows the struct
//!    together with every impl written for it.
//! 3. An `impl` block whose **trait** is that path, so `--item AreaCalculator` shows a component's
//!    provider-trait impls. This is what makes the filter useful on CGP code: the interesting part of
//!    a CGP expansion is impls, and the name a reader has in mind is usually the trait's.
//!
//! A path is matched against the item's own written path, qualified by the module it sits in — so
//! `shapes::Rectangle` finds a `Rectangle` declared or implemented inside `mod shapes`.

use syn::{File, Item, Path, Type};

/// A `::`-separated path naming what to expand, such as `shapes::Rectangle`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemPath {
    segments: Vec<String>,
}

impl ItemPath {
    /// Parse a path pattern, or `None` when it is not one: after the crate-root prefix below, every
    /// segment must be a plain identifier and at least one must remain. Declining here rather than
    /// matching nothing keeps a typo (`shapes:Rectangle`, `shapes::`) from looking like an item that
    /// does not exist.
    ///
    /// A **crate-root prefix is accepted and dropped**: `crate::contexts::app`, `::contexts::app`, and
    /// `self::contexts::app` all mean `contexts::app`. Matching is against module paths within the
    /// crate being expanded, which carry no such prefix — but `crate::…` is how the module is spelled
    /// in the source, so it is what a reader reaches for, and rejecting it would be pedantry.
    pub fn parse(pattern: &str) -> Option<Self> {
        let mut segments: Vec<String> = pattern.split("::").map(str::to_owned).collect();

        // A leading `::` leaves an empty first segment; `crate` and `self` are real segments to drop.
        if segments
            .first()
            .is_some_and(|first| first.is_empty() || first == "crate" || first == "self")
        {
            segments.remove(0);
        }

        let plain = |segment: &String| {
            let mut chars = segment.chars();
            chars.next().is_some_and(|c| c == '_' || c.is_alphabetic())
                && chars.all(|c| c == '_' || c.is_alphanumeric())
        };

        (!segments.is_empty() && segments.iter().all(plain)).then_some(Self { segments })
    }
}

impl std::fmt::Display for ItemPath {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.segments.join("::"))
    }
}

/// Narrow `file` to the items `path` names, reporting whether anything matched.
///
/// On no match the file is left **empty**, not unchanged: an expansion of the whole crate is not what
/// was asked for, and the caller reports the miss.
pub fn select_items(file: &mut File, path: &ItemPath) -> bool {
    let mut selected = Vec::new();
    collect(&file.items, &mut Vec::new(), path, &mut selected);

    let matched = !selected.is_empty();
    file.items = selected;
    // The crate's own attributes describe the whole crate, so they are dropped along with the items
    // they no longer accompany.
    file.attrs.clear();
    matched
}

/// Walk `items`, pushing everything `path` selects. `prefix` is the module path the items sit in.
fn collect(items: &[Item], prefix: &mut Vec<String>, path: &ItemPath, out: &mut Vec<Item>) {
    for item in items {
        if let Some(name) = declared_name(item)
            && matches(path, prefix, std::slice::from_ref(&name))
        {
            // A module selects its contents; anything else selects itself.
            match item {
                Item::Mod(module) => {
                    if let Some((_, inner)) = &module.content {
                        out.extend(inner.iter().cloned());
                    }
                }
                other => out.push(other.clone()),
            }
            continue;
        }

        match item {
            // An impl has no name of its own, so it is selected by what it is *about*.
            Item::Impl(item_impl) => {
                let self_ty = self_type_path(&item_impl.self_ty);
                let trait_path = item_impl.trait_.as_ref().map(|(_, path, _)| path);
                let selects = self_ty.is_some_and(|path_ref| matches_path(path, prefix, path_ref))
                    || trait_path.is_some_and(|path_ref| matches_path(path, prefix, path_ref));
                if selects {
                    out.push(item.clone());
                }
            }
            // Descend into a module whose path the pattern reaches through.
            Item::Mod(module) => {
                if let Some((_, inner)) = &module.content {
                    prefix.push(module.ident.to_string());
                    collect(inner, prefix, path, out);
                    prefix.pop();
                }
            }
            _ => {}
        }
    }
}

/// Whether `path` names the item written as `written` inside module `prefix`.
///
/// Two spellings count: the item's name qualified by the module it sits in (`shapes::Rectangle` for a
/// `Rectangle` inside `mod shapes`), and the path exactly as written (`shapes::Rectangle` for an impl
/// that spells its self type out that way).
fn matches(path: &ItemPath, prefix: &[String], written: &[String]) -> bool {
    let qualified: Vec<&String> = prefix.iter().chain(written.iter()).collect();
    let as_written: Vec<&String> = written.iter().collect();
    let target: Vec<&String> = path.segments.iter().collect();

    qualified == target || as_written == target
}

/// [`matches`] over a `syn::Path`, comparing by its last segment as written (qualified by `prefix`)
/// and by its whole spelling.
fn matches_path(path: &ItemPath, prefix: &[String], written: &Path) -> bool {
    let segments: Vec<String> = written
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect();
    let Some(last) = segments.last() else {
        return false;
    };

    matches(path, prefix, std::slice::from_ref(last)) || matches(path, &[], &segments)
}

/// The path an impl's self type names, or `None` when it is not a plain path type (a reference, a
/// tuple, a generic parameter's projection — none of which a reader names as an item).
fn self_type_path(self_ty: &Type) -> Option<&Path> {
    match self_ty {
        Type::Path(path) if path.qself.is_none() => Some(&path.path),
        _ => None,
    }
}

/// The name an item declares, when it declares one. An impl has none, which is why the selection
/// falls back to what the impl is about.
fn declared_name(item: &Item) -> Option<String> {
    let ident = match item {
        Item::Const(item) => &item.ident,
        Item::Enum(item) => &item.ident,
        Item::ExternCrate(item) => &item.ident,
        Item::Fn(item) => &item.sig.ident,
        Item::Macro(item) => return item.ident.as_ref().map(ToString::to_string),
        Item::Mod(item) => &item.ident,
        Item::Static(item) => &item.ident,
        Item::Struct(item) => &item.ident,
        Item::Trait(item) => &item.ident,
        Item::TraitAlias(item) => &item.ident,
        Item::Type(item) => &item.ident,
        Item::Union(item) => &item.ident,
        _ => return None,
    };
    Some(ident.to_string())
}
