//! The `cgp-field` crate and the field-access and extensible-data items it defines.

/// The getter provider that reads one named field, defined by [`CGP_FIELD_CRATE`].
pub const USE_FIELD_TYPE: &str = "UseField";

/// The item name of the field-access trait whose unmet bound is the root cause the typed
/// resolver reports, paired with [`CGP_FIELD_CRATE`] to confirm a leaf obligation is a
/// genuine CGP `HasField` before decoding its `Symbol!` field name.
pub const HAS_FIELD_TRAIT: &str = "HasField";

/// The crate that defines [`HAS_FIELD_TRAIT`] — distinct from
/// [`CGP_COMPONENT_CRATE`](super::CGP_COMPONENT_CRATE), since `HasField` lives in `cgp-field`, not
/// `cgp-component`.
pub const CGP_FIELD_CRATE: &str = "cgp_field";

/// The name of the type-level sum-list cell (defined by [`CGP_FIELD_CRATE`]): a `Sum!` expands to a
/// right-nested `Either<Head, Tail>` spine ended by [`VOID_TYPE`]. The dependency-tree renderer
/// resugars such a spine back to `Sum![…]`, anchored by `DefId` to `cgp-field`.
pub const EITHER_TYPE: &str = "Either";

/// The name of the type-level empty-sum terminator (defined by [`CGP_FIELD_CRATE`]) that closes an
/// `Either` spine — the sum counterpart of [`NIL_TYPE`](super::NIL_TYPE).
pub const VOID_TYPE: &str = "Void";

/// The name of the type-level lifetime lift (defined by [`CGP_FIELD_CRATE`]): `Life<'a>` stands in
/// for the lifetime `'a` wherever a component's parameters must all be types — the `Params` slot of
/// `CanUseComponent`/`IsProviderFor` and a `check_components!` entry. The resolver reads the region
/// back out of it when it rebuilds a real consumer obligation, whose trait wants the lifetime in a
/// region slot, not a type standing in for one.
pub const LIFE_TYPE: &str = "Life";

/// The name of the type-level field cell (defined by [`CGP_FIELD_CRATE`]): `Field<Tag, Value>`
/// pairs a `Symbol!` name tag or an [`INDEX_TYPE`] position tag with a value type. When every
/// element of a resugared `Product!`/`Sum!` list is a `Field`, the dependency-tree renderer
/// resugars the whole list to the CGP `Struct!`/`Enum!` shape it describes, where the shape has an
/// exact spelling. Anchored by `DefId` to `cgp-field`.
pub const FIELD_TYPE: &str = "Field";

/// The name of the type-level position tag (defined by [`CGP_FIELD_CRATE`]): `Index<N>` tags the
/// `N`th field of a tuple struct or tuple variant inside a [`FIELD_TYPE`] cell, and is what lets a
/// field list resugar to the tuple form `Struct!(A, B)`.
pub const INDEX_TYPE: &str = "Index";

/// The trait (defined by [`CGP_FIELD_CRATE`]) every partial record or enum a data derive generates
/// implements, naming the struct or enum it builds or matches as its `Target`. Its impl is what
/// marks `__PartialPerson<…>` as a partial value rather than an ordinary type.
pub const PARTIAL_DATA_TRAIT: &str = "PartialData";

/// The associated type of [`PARTIAL_DATA_TRAIT`] that names a partial value's struct or enum.
pub const PARTIAL_DATA_TARGET: &str = "Target";

/// The presence-marker trait (defined by [`CGP_FIELD_CRATE`]): a partial value's field or variant
/// is stored as `<M as MapType>::Map<T>`, so its marker parameter `M` records the field's state.
pub const MAP_TYPE_TRAIT: &str = "MapType";

/// The presence markers (defined by [`CGP_FIELD_CRATE`]): a field held (`IsPresent`), not yet set
/// (`IsNothing`), ruled out of an extractor (`IsVoid`), or optional (`IsOptional`).
pub const IS_PRESENT_TYPE: &str = "IsPresent";

/// See [`IS_PRESENT_TYPE`].
pub const IS_NOTHING_TYPE: &str = "IsNothing";

/// See [`IS_PRESENT_TYPE`].
pub const IS_VOID_TYPE: &str = "IsVoid";

/// See [`IS_PRESENT_TYPE`].
pub const IS_OPTIONAL_TYPE: &str = "IsOptional";

/// The borrow markers (defined by [`CGP_FIELD_CRATE`]) a reference extractor's partial enum carries
/// first, saying whether it holds shared (`IsRef`) or mutable (`IsMut`) references to its payloads.
pub const IS_REF_TYPE: &str = "IsRef";

/// See [`IS_REF_TYPE`].
pub const IS_MUT_TYPE: &str = "IsMut";

/// The extensible-data traits (defined by [`CGP_FIELD_CRATE`]) whose unmet obligation the data
/// leaf reads: a builder's per-field update and finalize, an extractor's per-variant extraction and
/// finalize, and an enum's construction by variant.
pub const UPDATE_FIELD_TRAIT: &str = "UpdateField";

/// See [`UPDATE_FIELD_TRAIT`].
pub const FINALIZE_BUILD_TRAIT: &str = "FinalizeBuild";

/// See [`UPDATE_FIELD_TRAIT`].
pub const EXTRACT_FIELD_TRAIT: &str = "ExtractField";

/// See [`UPDATE_FIELD_TRAIT`].
pub const FINALIZE_EXTRACT_TRAIT: &str = "FinalizeExtract";

/// See [`UPDATE_FIELD_TRAIT`].
pub const FROM_VARIANT_TRAIT: &str = "FromVariant";

/// The associated type of [`UPDATE_FIELD_TRAIT`] that names a field's state before the update, and
/// of [`FROM_VARIANT_TRAIT`] that names a variant's payload.
pub const MAPPER_ASSOC: &str = "Mapper";

/// See [`MAPPER_ASSOC`].
pub const VALUE_ASSOC: &str = "Value";

/// The field-access traits (defined by [`CGP_FIELD_CRATE`]) beside its extensible-data
/// operations: reading or writing one field of a value by its tag, which is not an operation on a
/// whole record or enum.
pub const FIELD_ACCESS_TRAITS: &[&str] =
    &["HasField", "HasFieldMut", "FieldGetter", "MutFieldGetter"];

/// Each extensible-data operation on a struct or enum value paired with its counterpart on a
/// partial value, both defined by [`CGP_FIELD_CRATE`]: a downcast of an enum, and a downcast of an
/// extractor (the remainder of an earlier downcast).
pub const PARTIAL_VALUE_OPERATIONS: &[(&str, &str)] = &[("CanDowncast", "CanDowncastFields")];

/// Each extensible-data trait a type implements only through a derive, paired with that derive:
/// the traits the data leaf reports as unimplemented on a struct or enum, naming the derive to add.
/// All are defined by [`CGP_FIELD_CRATE`].
pub const DATA_DERIVES: &[(&str, &str)] = &[
    ("HasFields", "HasFields"),
    ("HasFieldsRef", "HasFields"),
    ("HasExtractor", "ExtractField"),
    ("HasExtractorRef", "ExtractField"),
    ("HasExtractorMut", "ExtractField"),
    ("HasBuilder", "BuildField"),
    ("IntoBuilder", "BuildField"),
    ("FromVariant", "FromVariant"),
];
