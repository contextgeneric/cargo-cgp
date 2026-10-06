use crate::shape::shape_name;

/// The tag of one `Field` cell in a shape: a `Symbol!` name, or an `Index<N>` position.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShapeTag {
    /// A field or variant name, decoded from a `Symbol!` tag.
    Name(String),
    /// A tuple position, decoded from an `Index<N>` tag.
    Index(usize),
}

/// Render a product of `Field` cells, given as `(tag, rendered value)` pairs, as `Struct!`, or
/// `None` when the list has no exact `Struct!` spelling.
///
/// Every tag a name, each writable by [`shape_name`], gives the named form `Struct! { a: A }`.
/// Every tag an `Index`, numbered from 0 in order with at least two cells, gives the tuple form
/// `Struct!(A, B)`. A single `Index` cell has no spelling, because `Struct!(T)` is the bare `T`, and
/// neither has a list that mixes the two tags or skips a position.
pub fn render_struct_shape(entries: &[(ShapeTag, String)]) -> Option<String> {
    if entries.is_empty() {
        return None;
    }

    if let Some(body) = named_body(entries) {
        return Some(format!("Struct! {{ {body} }}"));
    }

    tuple_body(entries).map(|body| format!("Struct!({body})"))
}

/// The `a: A, b: B` body of a product whose tags are all writable names.
fn named_body(entries: &[(ShapeTag, String)]) -> Option<String> {
    let fields = entries
        .iter()
        .map(|(tag, value)| match tag {
            ShapeTag::Name(name) => Some(format!("{}: {value}", shape_name(name)?)),
            ShapeTag::Index(_) => None,
        })
        .collect::<Option<Vec<_>>>()?;

    Some(fields.join(", "))
}

/// The `A, B` body of a product tagged `Index<0>`, `Index<1>`, … in order, with at least two cells.
fn tuple_body(entries: &[(ShapeTag, String)]) -> Option<String> {
    if entries.len() < 2 {
        return None;
    }

    let types = entries
        .iter()
        .enumerate()
        .map(|(position, (tag, value))| match tag {
            ShapeTag::Index(index) if *index == position => Some(value.as_str()),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;

    Some(types.join(", "))
}
