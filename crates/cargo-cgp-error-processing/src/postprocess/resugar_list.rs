//! Post-processor: resugar the `Product!`/`Sum!` type-level lists and their `Struct!`/`Enum!` forms.
//!
//! `Product![A, B]` expands to the right-nested product spine `Cons<A, Cons<B, Nil>>`, and `Sum![A, B]`
//! to the sum spine `Either<A, Either<B, Void>>` (see the CGP `Product!` / `Sum!` references). This
//! reverses those spines back to the surface macro. A spine whose elements are *all* `Field` cells
//! resugars one step further to the CGP `Struct!` or `Enum!` shape it describes, when the shape has
//! an exact spelling: a product to `Struct! { name: Type, … }` or `Struct!(A, B)`, and a sum to
//! `Enum! { Name(Type), … }`. The spelling rules are the shared [`shape`](crate::shape) module's.
//!
//! This is the **fallback** counterpart of the driver's typed `render_ty` (which resugars the same
//! spines in the dependency tree, anchored by `DefId` to the CGP crates). It exists to catch a raw
//! spine in a diagnostic the resolver *declined* and left to rustc's own text, so a fallback message
//! reads the same as a reshaped one. Running on plain text, it cannot check the defining crate, so —
//! like every resugaring post-processor — it resugars only on an **exact structural match**: the spine
//! must close on the exact `Nil` (product) or `Void` (sum) terminator, and any other shape is left
//! untouched. It runs after [`resugar_symbol`](super::resugar_symbol), so a `Field`'s tag is already
//! in its `Symbol!("…")` surface form when a field/variant name is read from it, and each element is
//! resugared recursively so a nested list resugars in turn.

use crate::shape::{ShapeTag, render_enum_shape, render_struct_shape};

/// Which type-level list a spine is — the cell/terminator names it is built from and the surface
/// macros it resugars to (the plain list, and the record/variant form when every element is a field).
#[derive(Clone, Copy)]
enum ListKind {
    /// `Cons<…, Nil>` — a `Product!`, or a `Struct!` when every element is a `Field` cell.
    Product,
    /// `Either<…, Void>` — a `Sum!`, or an `Enum!` when every element is a named `Field` cell.
    Sum,
}

impl ListKind {
    /// The spine cell type name (`Cons` / `Either`).
    fn cell(self) -> &'static str {
        match self {
            ListKind::Product => "Cons",
            ListKind::Sum => "Either",
        }
    }

    /// The spine terminator type name (`Nil` / `Void`).
    fn terminator(self) -> &'static str {
        match self {
            ListKind::Product => "Nil",
            ListKind::Sum => "Void",
        }
    }

    /// The plain list macro used when the elements are not all named fields.
    fn list_macro(self) -> &'static str {
        match self {
            ListKind::Product => "Product!",
            ListKind::Sum => "Sum!",
        }
    }
}

/// Rewrite every well-formed `Cons`/`Either` spine in `text` to its surface form, returning the
/// rewritten text when any was rewritten (and `None` otherwise). A spine that does not close on its
/// exact terminator is emitted unchanged and scanning resumes past its opening token.
pub fn resugar_lists(text: &str) -> Option<String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut changed = false;

    while let Some((index, kind)) = next_cell(rest) {
        out.push_str(&rest[..index]);
        let candidate = &rest[index..];
        let token_len = kind.cell().len() + 1; // `Cons<` / `Either<`
        match parse_spine(candidate, kind) {
            Some((rendered, consumed)) => {
                out.push_str(&rendered);
                rest = &candidate[consumed..];
                changed = true;
            }
            None => {
                out.push_str(&candidate[..token_len]);
                rest = &candidate[token_len..];
            }
        }
    }

    out.push_str(rest);
    changed.then_some(out)
}

/// Find the next standalone `Cons<` or `Either<` in `text` — the start of a spine to try. The
/// preceding character must not be part of an identifier, so a `Cons<` inside `PathCons<` (or any
/// name ending in the cell's) is not mistaken for a spine cell.
fn next_cell(text: &str) -> Option<(usize, ListKind)> {
    let mut prev: Option<char> = None;
    for (index, ch) in text.char_indices() {
        if prev.is_none_or(|p| !is_ident_char(p)) {
            let rest = &text[index..];
            if rest.starts_with("Cons<") {
                return Some((index, ListKind::Product));
            }
            if rest.starts_with("Either<") {
                return Some((index, ListKind::Sum));
            }
        }
        prev = Some(ch);
    }
    None
}

/// Parse a `Cons`/`Either` spine at the start of `input` (which begins with the cell's opening token)
/// and render it as its surface form, returning that and the bytes consumed. `None` on any spine that
/// is not the cell all the way down to its exact terminator, or whose tail cell is not the whole
/// remaining tail (a malformed or open-ended spine, which must not be silently rewritten).
fn parse_spine(input: &str, kind: ListKind) -> Option<(String, usize)> {
    let open = &input[..kind.cell().len() + 1];
    let after = input.strip_prefix(open)?;
    let (head, tail, inner_len) = scan_head_tail(after)?;
    let consumed = open.len() + inner_len;

    let mut elems = vec![head.trim()];
    let mut tail = tail.trim();
    let mut guard = 0;
    loop {
        guard += 1;
        if guard > 4096 {
            return None;
        }
        if tail == kind.terminator() {
            break;
        }
        let tail_after = tail.strip_prefix(open)?;
        let (tail_head, tail_tail, tail_inner_len) = scan_head_tail(tail_after)?;
        // The tail's cell must be the whole tail — nothing may trail it, or the spine is malformed.
        if open.len() + tail_inner_len != tail.len() {
            return None;
        }
        elems.push(tail_head.trim());
        tail = tail_tail.trim();
    }

    Some((render(kind, &elems), consumed))
}

/// Render a spine's element list as its surface form: the `Struct!`/`Enum!` shape when every element
/// is a `Field` cell and the shape has an exact spelling, otherwise the plain list macro. Each
/// element (or field value) is resugared recursively so a nested spine resugars in turn.
fn render(kind: ListKind, elems: &[&str]) -> String {
    if let Some(fields) = field_cells(elems) {
        let shape = match kind {
            ListKind::Product => render_struct_shape(&fields),
            ListKind::Sum => render_enum_shape(
                &fields
                    .into_iter()
                    .map(|(tag, value)| (tag, (value != "Nil").then_some(value)))
                    .collect::<Vec<_>>(),
            ),
        };
        if let Some(shape) = shape {
            return shape;
        }
    }
    let body = elems
        .iter()
        .map(|elem| resugar_element(elem))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{}[{body}]", kind.list_macro())
}

/// Interpret every element as a `Field<Tag, Value>` cell, returning each tag and rendered value, or
/// `None` if any element is not such a cell, so the caller keeps the plain list form. The value is
/// resugared recursively.
fn field_cells(elems: &[&str]) -> Option<Vec<(ShapeTag, String)>> {
    elems
        .iter()
        .map(|elem| {
            let elem = elem.trim();
            let after = elem.strip_prefix("Field<")?;
            let (tag, value, inner_len) = scan_head_tail(after)?;
            // `Field<…>` must be the whole element — a trailing `::Something` or arguments mean it is
            // not a bare field cell.
            if "Field<".len() + inner_len != elem.len() {
                return None;
            }
            Some((shape_tag(tag.trim())?, resugar_element(value.trim())))
        })
        .collect()
}

/// Recursively resugar a single element's text, so a nested `Cons`/`Either` spine inside it becomes
/// its own surface form; text with no spine is returned unchanged.
fn resugar_element(elem: &str) -> String {
    resugar_lists(elem).unwrap_or_else(|| elem.to_owned())
}

/// The tag of a `Field` cell: the name inside a `Symbol!("name")` tag, or the position inside an
/// `Index<N>` tag. `None` for any other tag. Only an unescaped symbol literal is read, so a name that
/// cannot be read back verbatim leaves the list as its plain form rather than being decoded by
/// guesswork. Meant to run after [`resugar_symbol`](super::resugar_symbol) has already produced the
/// `Symbol!("…")` surface form.
fn shape_tag(tag: &str) -> Option<ShapeTag> {
    if let Some(index) = tag
        .strip_prefix("Index<")
        .and_then(|rest| rest.strip_suffix('>'))
    {
        if !index.is_empty() && index.chars().all(|c| c.is_ascii_digit()) {
            return index.parse().ok().map(ShapeTag::Index);
        }
        return None;
    }
    let inner = tag.strip_prefix("Symbol!(\"")?.strip_suffix("\")")?;
    if inner.contains('"') || inner.contains('\\') {
        return None;
    }
    Some(ShapeTag::Name(inner.to_owned()))
}

/// Split the content after a cell's opening token into its `Head` and `Tail`, returning them and the
/// byte offset just past the matching `>`. Angle, paren, and bracket nesting is balanced and string
/// literals are skipped, so a comma or `>` inside an element does not mislead the scan. `None` when
/// there is no top-level comma or no matching close.
fn scan_head_tail(s: &str) -> Option<(&str, &str, usize)> {
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

/// Whether `c` can appear inside a Rust identifier — used to keep `next_cell` from matching a cell
/// name that is only the tail of a longer identifier (`PathCons`).
fn is_ident_char(c: char) -> bool {
    c == '_' || c.is_alphanumeric()
}
