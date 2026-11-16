//! Taking the `--item <path>` filter out of the arguments bound for cargo.

use anyhow::bail;

use crate::config::ITEM_FLAG;
use crate::launch::scan_flag;

/// Split the forwarded arguments into the ones cargo gets and the `--item` path, removing
/// `--item <path>` and `--item=<path>`; a bare positional path is not accepted. See
/// `cgp-knowledge-base/cargo-cgp/implementation/expand-command.md`.
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

/// Record the path, rejecting an empty one and a second `--item`, since either would leave what
/// gets expanded ambiguous, and guessing is worse than asking.
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

/// Whether `path` is a `::`-separated run of plain identifiers, optionally rooted at the crate,
/// checked before anything compiles; see
/// `cgp-knowledge-base/cargo-cgp/implementation/expand-command.md`.
fn is_item_path(path: &str) -> bool {
    let path = path.strip_prefix("::").unwrap_or(path);
    !path.is_empty()
        && path.split("::").all(|segment| {
            let mut chars = segment.chars();
            chars.next().is_some_and(|c| c == '_' || c.is_alphabetic())
                && chars.all(|c| c == '_' || c.is_alphanumeric())
        })
}
