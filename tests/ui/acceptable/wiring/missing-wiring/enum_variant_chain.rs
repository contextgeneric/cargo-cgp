//! Acceptable: a missing wiring reached through a sum spine of *named* variants, whose
//! dependency-tree entries resugar `Either<Field<…>, …, Void>` all the way to the `Enum! { … }`
//! surface form.
//!
//! `EncodeChoice` dispatches over the variant list
//! `Sum![Field<Symbol!("Rect"), u64>, Field<Symbol!("Circle"), f64>]` — the shape an enum's
//! `HasFields` produces, written out here directly — through the `EncodeVariants` visitor, each
//! variant's payload encoded via the context. `App` wires `u64` but not `f64`, so the
//! `check_components!` entry for `Choice` fails on the missing `@VariantEncoderComponent.f64` wiring,
//! reached by following the sum spine.
//!
//! The point of the fixture is the rendering: because every element of the sum is a
//! `Field<Symbol!("Name"), Type>`, the renderer resugars the whole list past `Sum![…]` to
//! `Enum! { Rect(u64), Circle(f64) }`, and its tail to `Enum! { Circle(f64) }` — the enum the
//! variant list represents. It is the sum counterpart of `record_field_chain`'s `Struct! { … }`.
//! (`Struct!`/`Enum!` are presentation-only forms, not real CGP macros.)

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

// A recursive visitor over a sum spine of *named* variants `Either<Field<Tag, Value>, Tail>`,
// encoding each variant's payload through the context.
pub trait EncodeVariants<Context> {
    fn encode_variants(context: &Context) -> String;
}

impl<Context, Tag, Value, Tail> EncodeVariants<Context> for Either<Field<Tag, Value>, Tail>
where
    Context: CanEncodeVariant<Value>,
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
    Sum![Field<Symbol!("Rect"), u64>, Field<Symbol!("Circle"), f64>]: EncodeVariants<Self>,
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
        // `f64` (the `Circle` payload) is deliberately left unwired — the mistake this fixture pins,
        // reached through the variant list `Enum! { Rect(u64), Circle(f64) }`.
    }
}

check_components! {
    App {
        VariantEncoderComponent: [Choice],
    }
}

fn main() {}
