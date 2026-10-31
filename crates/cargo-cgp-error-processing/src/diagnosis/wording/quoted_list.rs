//! The `and`-joined list the cause sentences share.

/// An `and`-joined, back-quoted list: `` `x` ``, `` `x` and `y` ``, or `` `x`, `y`, and `z` ``.
pub fn quoted_list(items: &[String]) -> String {
    let quoted: Vec<String> = items.iter().map(|item| format!("`{item}`")).collect();
    match quoted.as_slice() {
        [] => String::new(),
        [one] => one.clone(),
        [a, b] => format!("{a} and {b}"),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
    }
}
