//! Taking the `--item <path>` filter out of the arguments bound for cargo.

use anyhow::bail;

/// The flag that names one module or item to expand.
pub const ITEM_FLAG: &str = "--item";

/// Split the forwarded arguments into the ones cargo gets and the item path, if one was given.
///
/// Recognizes `--item <path>` and `--item=<path>`, and **removes** them, since cargo knows no such
/// flag. The filter is the front-end's one tool-specific argument; everything else still passes
/// through verbatim.
///
/// A bare positional path (`cargo cgp expand my::module`, the way `cargo-expand` takes it) is
/// deliberately not accepted: with every other argument forwarded untouched, a bare word cannot be
/// told from the value of a cargo flag (`--bin my_module`) without re-declaring cargo's whole
/// argument grammar here — which is exactly what forwarding exists to avoid.
pub fn take_item(args: &[String]) -> anyhow::Result<(Vec<String>, Option<String>)> {
    let prefix = format!("{ITEM_FLAG}=");
    let mut forwarded = Vec::with_capacity(args.len());
    let mut item = None;
    let mut arguments = args.iter();

    while let Some(arg) = arguments.next() {
        if let Some(path) = arg.strip_prefix(&prefix) {
            set_item(&mut item, path.to_owned())?;
        } else if arg == ITEM_FLAG {
            let Some(path) = arguments.next() else {
                bail!("`{ITEM_FLAG}` needs the path of a module or item to expand");
            };
            set_item(&mut item, path.clone())?;
        } else {
            forwarded.push(arg.clone());
        }
    }

    Ok((forwarded, item))
}

/// Record the path, rejecting an empty one and a second `--item` — either would leave what gets
/// expanded ambiguous, and guessing is worse than asking.
fn set_item(item: &mut Option<String>, path: String) -> anyhow::Result<()> {
    if path.is_empty() {
        bail!("`{ITEM_FLAG}` needs the path of a module or item to expand");
    }
    if let Some(previous) = item {
        bail!(
            "`{ITEM_FLAG}` was given twice (`{previous}` and `{path}`); expand one path at a time"
        );
    }
    if !is_item_path(&path) {
        bail!(
            "`{path}` is not an item path: expected `::`-separated identifiers, as in `shapes::Rectangle`"
        );
    }

    *item = Some(path);
    Ok(())
}

/// Whether `path` is a `::`-separated run of plain identifiers, optionally rooted at the crate.
///
/// Checked here, before anything is compiled, so a typo fails at once rather than after a build that
/// then matches nothing. The driver parses the path again for real — that parser owns the matching and
/// the crate-root prefix (`crate::`, `self::`, a leading `::`) it strips — so this is a fail-fast
/// courtesy rather than the authority, and it only has to be no stricter.
fn is_item_path(path: &str) -> bool {
    let path = path.strip_prefix("::").unwrap_or(path);
    !path.is_empty()
        && path.split("::").all(|segment| {
            let mut chars = segment.chars();
            chars.next().is_some_and(|c| c == '_' || c.is_alphabetic())
                && chars.all(|c| c == '_' || c.is_alphanumeric())
        })
}
