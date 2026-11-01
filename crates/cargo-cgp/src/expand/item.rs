//! Taking the `--item <path>` filter out of the arguments bound for cargo.

use anyhow::bail;

use crate::config::ITEM_FLAG;
use crate::launch::scan_flag;

/// Split the forwarded arguments into the ones cargo gets and the item path, if one was given.
///
/// Recognizes `--item <path>` and `--item=<path>` among the arguments cargo reads, and **removes**
/// them, since cargo knows no such flag. The filter is the front-end's one tool-specific argument;
/// everything else still passes through verbatim, and an `--item` after a `--` is rustc's.
///
/// A bare positional path (`cargo cgp expand my::module`, the way `cargo-expand` takes it) is
/// deliberately not accepted: with every other argument forwarded untouched, a bare word cannot be
/// told from the value of a cargo flag (`--bin my_module`) without re-declaring cargo's whole
/// argument grammar here, which is exactly what forwarding exists to avoid.
pub fn take_item(args: &[String]) -> anyhow::Result<(Vec<String>, Option<String>)> {
    let scan = scan_flag(args, ITEM_FLAG);
    let mut item = None;
    for value in scan.values {
        let Some(path) = value else {
            bail!("`{ITEM_FLAG}` needs the path of a module or item to expand");
        };
        set_item(&mut item, path)?;
    }

    Ok((scan.remainder, item))
}

/// Record the path, rejecting an empty one and a second `--item`, since either would leave what gets
/// expanded ambiguous, and guessing is worse than asking.
fn set_item(item: &mut Option<String>, path: String) -> anyhow::Result<()> {
    if path.is_empty() {
        bail!("`{ITEM_FLAG}` needs the path of a module or item to expand");
    }
    if let Some(previous) = item {
        bail!(
            "`{ITEM_FLAG}` was given twice (`{previous}` and `{path}`)\n\n\
             help: expand one path at a time"
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
/// then matches nothing. The driver parses the path again for real: that parser owns the matching and
/// the crate-root prefix (`crate::`, `self::`, a leading `::`) it strips, so this is a fail-fast
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
