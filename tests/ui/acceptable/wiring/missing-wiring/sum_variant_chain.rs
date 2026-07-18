//! Acceptable: a missing wiring reached *through* a type-level sum spine, whose dependency-tree
//! entries resugar `Either<…, Void>` back to `Sum![…]`.
//!
//! `EncodeChoice` dispatches over the variant list `Sum![u64, f64]` — which expands to the sum
//! spine `Either<u64, Either<f64, Void>>` — through the recursive `EncodeVariants` visitor, each
//! variant encoded via the context. `App` wires `u64` but not `f64`, so the `check_components!`
//! entry for `Choice` fails on the missing `@VariantEncoderComponent.f64` wiring, reached by
//! following the sum spine's same-trait recursion (the sum counterpart of `record_field_chain`).
//!
//! The point of the fixture is the rendering: the tree entries for the `EncodeVariants` steps carry
//! the sum spine as their self type, and the renderer resugars it — `Either<u64, Either<f64, Void>>`
//! reads as `Sum![u64, f64]` and its tail as `Sum![f64]`, anchored by `DefId` to `cgp-field` so only
//! CGP's own `Either`/`Void` are resugared.

use cgp::prelude::*;

#[cgp_component(VariantEncoder)]
pub trait CanEncodeVariant<Value> {
    fn encode_variant(&self, value: &Value) -> String;
}

#[cgp_impl(new EncodeU64)]
impl VariantEncoder<u64> {
    fn encode_variant(&self, value: &u64) -> String {
        value.to_string()
    }
}

// A recursive visitor over the `Either`/`Void` sum spine: encodes the head variant *through the
// context* and recurses on the tail (the sum counterpart of a `Cons`/`Nil` field-list handler).
pub trait EncodeVariants<Context> {
    fn encode_variants(context: &Context) -> String;
}

impl<Context, Head, Tail> EncodeVariants<Context> for Either<Head, Tail>
where
    Context: CanEncodeVariant<Head>,
    Tail: EncodeVariants<Context>,
{
    fn encode_variants(_context: &Context) -> String {
        String::new()
    }
}

impl<Context> EncodeVariants<Context> for Void {
    fn encode_variants(_context: &Context) -> String {
        String::new()
    }
}

pub struct Choice;

#[cgp_impl(new EncodeChoice)]
impl VariantEncoder<Choice>
where
    Sum![u64, f64]: EncodeVariants<Self>,
{
    fn encode_variant(&self, _value: &Choice) -> String {
        String::new()
    }
}

pub struct App;

delegate_components! {
    App {
        open VariantEncoderComponent;

        @VariantEncoderComponent.Choice: EncodeChoice,
        @VariantEncoderComponent.u64: EncodeU64,
        // `f64` is deliberately left unwired — the mistake this fixture pins, reached through the
        // sum spine `Sum![u64, f64]`.
    }
}

check_components! {
    App {
        VariantEncoderComponent: [Choice],
    }
}

fn main() {}
