//! Choosing the version an update moves to.

use anyhow::Context;
use semver::Version;

/// Pick the highest version that `current`'s channel allows and that is strictly newer than
/// it, or `None` when there is nothing to move to. A stable current (no pre-release) considers
/// only stable candidates, so it never jumps to a pre-release. A pre-release current considers
/// every candidate, pre-release or stable: semver orders `1.1.0-beta` below `1.1.0`, so a
/// pre-release install reaches the release it previewed, and moves on from there.
pub fn select_update(versions: &[String], current: &str) -> anyhow::Result<Option<String>> {
    let current = Version::parse(current)
        .with_context(|| format!("`{current}` is not a valid semver version"))?;
    let accepts_prerelease = !current.pre.is_empty();

    let best = versions
        .iter()
        .filter_map(|version| Version::parse(version).ok())
        .filter(|version| accepts_prerelease || version.pre.is_empty())
        .filter(|version| *version > current)
        .max();

    Ok(best.map(|version| version.to_string()))
}
