//! The `cgp-base-types` crate and the type-level spines it defines.

/// The crate that defines the type-level string spine (`Symbol`, `Chars`, `Nil`) the resolver
/// walks to decode a field name. Anchoring the decode to this crate keeps a same-named type
/// from another crate from being mistaken for CGP's.
pub const CGP_BASE_TYPES_CRATE: &str = "cgp_base_types";

/// The name of the type-level path spine (defined by [`CGP_BASE_TYPES_CRATE`]) that `Path!`/`open`
/// wiring expands to. An unmet `DelegateComponent` whose key is a `PathCons` is a redirect *path*
/// the context does not terminate, not a bare component marker, so the leaf classifier renders the
/// whole path rather than reading only its ADT item name (`PathCons`).
pub const PATH_CONS_TYPE: &str = "PathCons";

/// The type-level string `Symbol!` expands to, in `cgp-base-types`, whose `Chars` spine ends in
/// `Nil` as a string terminator rather than an empty list.
pub const SYMBOL_TYPE: &str = "Symbol";

/// The character cell of a `Symbol`'s spine, in `cgp-base-types`.
pub const CHARS_TYPE: &str = "Chars";

/// The name of the type-level empty-path/empty-list terminator (defined by
/// [`CGP_BASE_TYPES_CRATE`]) that closes a `PathCons` spine. The use-site resolver reads it to
/// recognize a simple two-segment `open`-dispatch key `PathCons<Component, PathCons<Value, Nil>>`,
/// so it can recover the real dispatch parameter `Value` rather than re-checking the raw path.
/// It is also the terminator of a `Product!` / `Cons` list, which the tree renderer resugars.
pub const NIL_TYPE: &str = "Nil";

/// The name of the type-level product-list cell (defined by [`CGP_BASE_TYPES_CRATE`]): a `Product!`
/// expands to a right-nested `Cons<Head, Tail>` spine ended by [`NIL_TYPE`]. The dependency-tree
/// renderer resugars such a spine back to `Product![…]`, anchored by `DefId` to this crate so a
/// same-named type is never resugared.
pub const CONS_TYPE: &str = "Cons";
