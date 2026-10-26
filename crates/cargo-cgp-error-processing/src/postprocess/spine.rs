/// Split the content after a cell's opening token (`Cons<`, `Either<`, `PathCons<`) into its `Head`
/// and `Tail`, returning them and the byte offset just past the matching `>`. Angle, paren, and
/// bracket nesting is balanced and string literals are skipped, so a comma or `>` inside an element
/// does not mislead the scan. `None` when there is no top-level comma or no matching close.
pub fn scan_head_tail(s: &str) -> Option<(&str, &str, usize)> {
    let mut depth: i32 = 0;
    let mut comma: Option<usize> = None;
    let mut in_string = false;
    let mut escaped = false;

    for (index, c) in s.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' => in_string = true,
            '<' | '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            '>' if depth == 0 => {
                let comma = comma?;
                return Some((&s[..comma], &s[comma + 1..index], index + 1));
            }
            '>' => depth -= 1,
            ',' if depth == 0 && comma.is_none() => comma = Some(index),
            _ => {}
        }
    }
    None
}
