//! Folding the `help` facts of one cause reached by several routes.

use crate::diagnosis::leaf::Leaf;

impl Leaf {
    /// Fold the `help` facts of `other`, the same cause reached another way, into this leaf: adding
    /// a field stays confirmed only when both routes confirm it, and a type or a near miss is kept
    /// only when both routes name the same one, since a help naming one route's would not fix the
    /// other.
    pub fn reconcile_fix(&mut self, other: &Leaf) {
        fn agree(mine: &mut Option<String>, theirs: &Option<String>) {
            if mine != theirs {
                *mine = None;
            }
        }
        match (self, other) {
            (
                Leaf::NotAProvider {
                    adapter,
                    provides,
                    core,
                    ..
                },
                Leaf::NotAProvider {
                    adapter: their_adapter,
                    provides: their_provides,
                    core: their_core,
                    ..
                },
            ) => {
                if adapter.0 != their_adapter.0 {
                    adapter.0.clear();
                }
                if provides.0 != their_provides.0 {
                    provides.0.clear();
                }
                if core.0 != their_core.0 {
                    core.0 = None;
                }
            }
            (Leaf::Field { fix, .. }, Leaf::Field { fix: theirs, .. }) => {
                fix.confirmed &= theirs.confirmed;
                agree(&mut fix.ty, &theirs.ty);
                agree(&mut fix.similar, &theirs.similar);
                agree(&mut fix.index_tag, &theirs.index_tag);
            }
            (
                Leaf::MissingWiring { similar, .. },
                Leaf::MissingWiring {
                    similar: theirs, ..
                },
            )
            | (
                Leaf::MissingRedirectWiring { similar, .. },
                Leaf::MissingRedirectWiring {
                    similar: theirs, ..
                },
            )
            | (
                Leaf::MissingVariant { similar, .. },
                Leaf::MissingVariant {
                    similar: theirs, ..
                },
            ) => agree(&mut similar.0, &theirs.0),
            (
                Leaf::DataTraitUnimplemented { derive, .. },
                Leaf::DataTraitUnimplemented { derive: theirs, .. },
            ) => agree(&mut derive.0, &theirs.0),
            (
                Leaf::WiringWithoutProviderFor { delegate, .. },
                Leaf::WiringWithoutProviderFor {
                    delegate: theirs, ..
                },
            ) => agree(&mut delegate.0, &theirs.0),
            (
                Leaf::FieldNotOptional { convert: fix, .. },
                Leaf::FieldNotOptional {
                    convert: theirs, ..
                },
            )
            | (
                Leaf::PartialLacksDataTrait {
                    replacement: fix, ..
                },
                Leaf::PartialLacksDataTrait {
                    replacement: theirs,
                    ..
                },
            ) if fix.0 != theirs.0 => fix.0 = None,
            _ => {}
        }
    }
}
