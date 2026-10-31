//! Finding a near miss: a field or wiring key the program wrote that is close to the one missing.

use crate::syntax::is_primitive_type;

/// The candidate closest to `target` when it is close enough to be a likely slip: a candidate equal
/// to `target` ignoring ASCII case, or one within an edit distance of a third of `target`'s length,
/// as rustc's own "similar name" suggestions allow, but never more than two edits, so names that
/// merely share a long stem stay apart. `target` itself is never a near miss of itself. Ties go to
/// the first candidate, so the result follows the caller's order.
pub fn similar_name<'a>(
    target: &str,
    candidates: impl IntoIterator<Item = &'a str>,
) -> Option<&'a str> {
    let limit = (target.chars().count() / 3).clamp(1, 2);
    let mut best: Option<(&str, usize)> = None;
    for candidate in candidates {
        if candidate == target {
            continue;
        }
        let distance = if candidate.eq_ignore_ascii_case(target) {
            0
        } else {
            edit_distance(candidate, target)
        };
        if distance <= limit && best.is_none_or(|(_, seen)| distance < seen) {
            best = Some((candidate, distance));
        }
    }
    best.map(|(candidate, _)| candidate)
}

/// The first candidate wiring key that is a likely slip of `target`: keys are `@`-paths
/// (`@app.GreeterComponent`) or bare component names (`GreeterComponent`, a path of one segment),
/// compared segment by segment. A candidate is a near miss when one segment differs, either by
/// ASCII case alone or, for two symbol segments, as a [`similar_name`]; or when it has one segment
/// more or fewer, other than the last, as a doubled or a missing prefix segment makes it. Two type
/// segments that differ otherwise are two different types, never a slip: a misspelled type does not
/// compile, so it cannot reach this point.
pub fn similar_key<'a>(
    target: &str,
    candidates: impl IntoIterator<Item = &'a str>,
) -> Option<&'a str> {
    let target_segments = segments(target);
    candidates
        .into_iter()
        .find(|candidate| is_near_key(&target_segments, &segments(candidate)))
}

/// Whether two keys, as segment lists, differ by one slipped segment.
fn is_near_key(target: &[&str], candidate: &[&str]) -> bool {
    if target == candidate {
        return false;
    }
    match target.len().abs_diff(candidate.len()) {
        0 => {
            let differing: Vec<(&str, &str)> = target
                .iter()
                .zip(candidate)
                .filter(|(a, b)| a != b)
                .map(|(a, b)| (*a, *b))
                .collect();
            matches!(differing.as_slice(), [(a, b)]
                if a.eq_ignore_ascii_case(b)
                    || (is_symbol(a) && is_symbol(b) && similar_name(a, [*b]).is_some()))
        }
        1 => {
            let (long, short) = if target.len() > candidate.len() {
                (target, candidate)
            } else {
                (candidate, target)
            };
            // The last segment names what is looked up, so dropping it is a different key.
            (0..long.len() - 1).any(|skip| {
                long.iter()
                    .enumerate()
                    .filter(|(at, _)| *at != skip)
                    .map(|(_, segment)| segment)
                    .eq(short.iter())
            })
        }
        _ => false,
    }
}

/// Whether a segment is a symbol, by `Path!`'s rule: it starts with a lowercase ASCII letter and is
/// not a primitive type, which stays a type (`u64` and `f64` are two types, not one slip).
fn is_symbol(segment: &str) -> bool {
    segment.starts_with(|c: char| c.is_ascii_lowercase()) && !is_primitive_type(segment)
}

/// A key's segments: the parts of an `@`-path between its dots, or a bare name as one segment.
fn segments(key: &str) -> Vec<&str> {
    key.strip_prefix('@').unwrap_or(key).split('.').collect()
}

/// The edit distance between two strings, counted in characters, with a swap of two adjacent
/// characters as one edit (the optimal-string-alignment distance rustc's own suggestions use), so
/// `frist` is one edit from `first`.
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    // `rows[i][j]` is the distance between the first `i` characters of `a` and the first `j` of `b`.
    let mut rows = vec![vec![0; b.len() + 1]; a.len() + 1];
    for (i, row) in rows.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in rows[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            let mut best = (rows[i - 1][j] + 1)
                .min(rows[i][j - 1] + 1)
                .min(rows[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                best = best.min(rows[i - 2][j - 2] + 1);
            }
            rows[i][j] = best;
        }
    }
    rows[a.len()][b.len()]
}
