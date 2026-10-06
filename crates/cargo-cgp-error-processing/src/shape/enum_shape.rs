use crate::shape::{ShapeTag, shape_name};

/// Render a sum of `Field` cells as `Enum!`, or `None` when the sum has no exact `Enum!` spelling.
///
/// Each entry is a variant's tag and its rendered payload, with `None` for a `Nil` payload. Every
/// tag must be a name writable by [`shape_name`], since a variant cannot be named by a position.
pub fn render_enum_shape(variants: &[(ShapeTag, Option<String>)]) -> Option<String> {
    if variants.is_empty() {
        return None;
    }

    let body = variants
        .iter()
        .map(|(tag, payload)| match tag {
            ShapeTag::Name(name) => Some(render_variant(&shape_name(name)?, payload.as_deref())),
            ShapeTag::Index(_) => None,
        })
        .collect::<Option<Vec<_>>>()?;

    Some(format!("Enum! {{ {} }}", body.join(", ")))
}

/// Render one variant in the shortest of the spellings that are all the same type.
///
/// An `Enum!` variant's fields are encoded by the `Struct!` rules, so a payload already rendered as
/// `Struct! { … }` is written as the variant's own braces, one rendered as `Struct!(…)` as its own
/// parentheses, and a `Nil` payload (`None`) as a unit variant. Any other payload is the single
/// positional field of `V(T)`.
pub fn render_variant(name: &str, payload: Option<&str>) -> String {
    let Some(payload) = payload else {
        return name.to_owned();
    };

    if let Some(body) = macro_body(payload, "Struct! {", '{', '}') {
        return format!("{name} {{ {} }}", body.trim());
    }

    if let Some(body) = macro_body(payload, "Struct!(", '(', ')') {
        return format!("{name}({body})");
    }

    format!("{name}({payload})")
}

/// The body of `text` when the whole of it is one call `prefix … close`, whose delimiter opened
/// at the end of `prefix` closes on the last character. A call followed by anything else, such as
/// `Struct!(A, B)::Assoc`, is not the whole text and gives `None`. Delimiters inside a string
/// literal, such as a `Symbol!("}")` tag, are skipped.
fn macro_body<'a>(text: &'a str, prefix: &str, open: char, close: char) -> Option<&'a str> {
    let body = text.strip_prefix(prefix)?.strip_suffix(close)?;

    let mut depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    for c in body.chars() {
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
        } else if c == '"' {
            in_string = true;
        } else if c == open {
            depth += 1;
        } else if c == close {
            depth -= 1;
            if depth < 0 {
                return None;
            }
        }
    }

    (depth == 0).then_some(body)
}
