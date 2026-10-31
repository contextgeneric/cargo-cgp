//! Reading the crates.io sparse index for a crate's published versions.

use anyhow::Context;

use crate::config::TOOL_VERSION;

/// Read the crates.io sparse index entry for `crate_name` over HTTP and return the body: one
/// JSON object per line, one per published version.
pub fn fetch_sparse_index(crate_name: &str) -> anyhow::Result<String> {
    let url = format!("https://index.crates.io/{}", sparse_index_path(crate_name));
    let user_agent = format!("cargo-cgp/{TOOL_VERSION}");

    ureq::get(url.as_str())
        .header("User-Agent", &user_agent)
        .call()
        .with_context(|| format!("failed to query the crates.io index at {url}"))?
        .body_mut()
        .read_to_string()
        .context("failed to read the crates.io index response")
}

/// The sparse-index path for a crate, following the registry's directory convention: one
/// segment for names of 1–2 characters, `3/<first>` for 3, and `<first two>/<next two>` for
/// 4 or more. Crate names are ASCII, so byte slicing is safe.
pub fn sparse_index_path(crate_name: &str) -> String {
    let name = crate_name.to_ascii_lowercase();
    let dir = match name.len() {
        0 => String::new(),
        1 => "1".to_owned(),
        2 => "2".to_owned(),
        3 => format!("3/{}", &name[0..1]),
        _ => format!("{}/{}", &name[0..2], &name[2..4]),
    };
    format!("{dir}/{name}")
}

/// Extract the non-yanked version strings from a sparse-index body. Each line is a JSON
/// object with `vers` and `yanked`; a line that does not parse or lacks `vers` is skipped,
/// and a yanked version is dropped so an update never lands on one.
pub fn parse_versions(index_body: &str) -> Vec<String> {
    index_body
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let value: serde_json::Value = serde_json::from_str(line).ok()?;
            if value.get("yanked").and_then(serde_json::Value::as_bool) == Some(true) {
                return None;
            }
            value
                .get("vers")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        })
        .collect()
}
