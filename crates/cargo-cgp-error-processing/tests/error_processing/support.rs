//! Builders for the values several test modules construct by hand: field leaves, trait hops, and
//! redirect hops. A module with fixed defaults wraps these in a shorter local helper.

use cargo_cgp_error_processing::{ChainNode, DepNode, FieldFix, FieldIssue, Leaf};

/// The leaf for `owner`'s `name` field, in the given state.
/// A missing field gets the usual fix: its struct derives `HasField` for its other fields, so
/// adding the field is confirmed, with no stated type.
pub fn field_leaf(name: &str, owner: &str, issue: FieldIssue) -> Leaf {
    let fix = FieldFix {
        confirmed: issue == FieldIssue::Missing,
        ..FieldFix::default()
    };
    Leaf::Field {
        name: name.to_owned(),
        owner: owner.to_owned(),
        issue,
        fix,
    }
}

/// A hop through an ordinary trait impl, `self_ty: trait_ref`.
pub fn trait_hop(trait_ref: &str, self_ty: &str) -> ChainNode {
    ChainNode::Hop(DepNode::Trait {
        trait_ref: trait_ref.to_owned(),
        self_ty: self_ty.to_owned(),
    })
}

/// A hop through a namespace redirect along `path`, looked up in `table` for `key`.
pub fn redirect(path: &str, table: &str, key: &str) -> ChainNode {
    ChainNode::Hop(DepNode::Redirect {
        path: path.to_owned(),
        table: table.to_owned(),
        key: key.to_owned(),
    })
}
