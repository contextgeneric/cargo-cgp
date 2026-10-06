//! Acceptable: a missing wiring reached through a sum of named variants that uses every variant
//! shape, whose dependency-tree entries render each variant in its shortest `Enum!` spelling.
//!
//! `EncodeChoice` dispatches over the variant list
//! `Enum! { Empty, Rect { w: f64 }, Pair(u8, u16), Circle(f64) }` through the `EncodeVariants`
//! visitor, encoding each variant's payload through the context. Under the `Struct!` rules those
//! payloads are `Nil`, `Struct! { w: f64 }`, `Struct!(u8, u16)`, and `f64`. `App` wires the first
//! three (keying two of them by a shape in its `open` entries) but not `f64`, so the
//! `check_components!` entry for `Choice` fails on the missing `@VariantEncoderComponent.f64`
//! wiring, reached by following the sum spine.
//!
//! The point of the fixture is the rendering. Each hop down the spine names the remaining variant
//! list as an `Enum!`, and each variant takes its shortest spelling: `Empty` for the `Nil` payload,
//! `Rect { w: f64 }` for the named-field payload, `Pair(u8, u16)` for the positional one, and
//! `Circle(f64)` for the bare payload. Each spelling is the same type as the field list it renders,
//! so the list can be copied back into code. It complements `enum_variant_chain`, whose variants
//! all carry a bare payload.

use cgp::prelude::*;

#[cgp_component(VariantEncoder)]
pub trait CanEncodeVariant<Value> {
    fn encode_variant(&self, value: &Value) -> String;
}

#[cgp_impl(new EncodeAny)]
impl<Value> VariantEncoder<Value> {
    fn encode_variant(&self, _value: &Value) -> String {
        String::new()
    }
}

// A recursive visitor over a sum spine of named variants `Either<Field<Tag, Value>, Tail>`,
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
    Enum! { Empty, Rect { w: f64 }, Pair(u8, u16), Circle(f64) }: EncodeVariants<Self>,
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
        @VariantEncoderComponent.Nil: EncodeAny,
        @VariantEncoderComponent.Struct! { w: f64 }: EncodeAny,
        @VariantEncoderComponent.Struct!(u8, u16): EncodeAny,
        // `f64` (the `Circle` payload) is deliberately left unwired: the mistake this fixture pins.
    }
}

check_components! {
    App {
        VariantEncoderComponent: [Choice],
    }
}

fn main() {}
