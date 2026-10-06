#![feature(prelude_import)]
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
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanEncodeVariant<Value> {
    fn encode_variant(&self, value: &Value) -> String;
}
impl<__Context__, Value> CanEncodeVariant<Value> for __Context__
where
    __Context__: VariantEncoder<__Context__, Value>,
{
    fn encode_variant(&self, value: &Value) -> String {
        __Context__::encode_variant(self, value)
    }
}
pub trait VariantEncoder<
    __Context__,
    Value,
>: IsProviderFor<VariantEncoderComponent, __Context__, (Value)> {
    fn encode_variant(__context__: &__Context__, value: &Value) -> String;
}
impl<__Provider__, __Context__, Value> VariantEncoder<__Context__, Value>
for __Provider__
where
    __Provider__: DelegateComponent<VariantEncoderComponent>
        + IsProviderFor<VariantEncoderComponent, __Context__, (Value)>,
    <__Provider__ as DelegateComponent<
        VariantEncoderComponent,
    >>::Delegate: VariantEncoder<__Context__, Value>,
{
    fn encode_variant(__context__: &__Context__, value: &Value) -> String {
        <__Provider__ as DelegateComponent<
            VariantEncoderComponent,
        >>::Delegate::encode_variant(__context__, value)
    }
}
pub struct VariantEncoderComponent;
impl<__Context__, Value> VariantEncoder<__Context__, Value> for UseContext
where
    __Context__: CanEncodeVariant<Value>,
{
    fn encode_variant(__context__: &__Context__, value: &Value) -> String {
        __Context__::encode_variant(__context__, value)
    }
}
impl<__Context__, Value> IsProviderFor<VariantEncoderComponent, __Context__, (Value)>
for UseContext
where
    __Context__: CanEncodeVariant<Value>,
{}
impl<__Context__, Value, __Components__, __Path__> VariantEncoder<__Context__, Value>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Value)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Value)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Value)>>::Output,
    >>::Delegate: VariantEncoder<__Context__, Value>,
{
    fn encode_variant(__context__: &__Context__, value: &Value) -> String {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@Value)>>::Output,
        >>::Delegate::encode_variant(__context__, value)
    }
}
impl<
    __Context__,
    Value,
    __Components__,
    __Path__,
> IsProviderFor<VariantEncoderComponent, __Context__, (Value)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Value)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Value)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Value)>>::Output,
    >>::Delegate: IsProviderFor<VariantEncoderComponent, __Context__, (Value)>
        + VariantEncoder<__Context__, Value>,
{}
impl<__Context__, Value> VariantEncoder<__Context__, Value> for EncodeAny {
    fn encode_variant(__context__: &__Context__, _value: &Value) -> String {
        String::new()
    }
}
impl<__Context__, Value> IsProviderFor<VariantEncoderComponent, __Context__, (Value)>
for EncodeAny {}
pub struct EncodeAny;
pub trait EncodeVariants<Context> {
    fn encode_variants(context: &Context) -> String;
}
impl<Context, Tag, Value, Tail> EncodeVariants<Context>
for Either<Field<Tag, Value>, Tail>
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
impl<__Context__> VariantEncoder<__Context__, Choice> for EncodeChoice
where
    Enum! { Empty, Rect { w: f64 }, Pair(u8, u16), Circle(f64) }: EncodeVariants<__Context__>,
{
    fn encode_variant(__context__: &__Context__, _value: &Choice) -> String {
        String::new()
    }
}
impl<__Context__> IsProviderFor<VariantEncoderComponent, __Context__, (Choice)>
for EncodeChoice
where
    Enum! { Empty, Rect { w: f64 }, Pair(u8, u16), Circle(f64) }: EncodeVariants<__Context__>,
{}
pub struct EncodeChoice;
pub struct App;
impl DelegateComponent<VariantEncoderComponent> for App {
    type Delegate = RedirectLookup<App, Path!(@VariantEncoderComponent)>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<VariantEncoderComponent, __Context__, __Params__> for App
where
    RedirectLookup<
        App,
        Path!(@VariantEncoderComponent),
    >: IsProviderFor<VariantEncoderComponent, __Context__, __Params__>,
{}
impl<
    __Wildcard__,
> DelegateComponent<PathCons<VariantEncoderComponent, PathCons<Choice, __Wildcard__>>>
for App {
    type Delegate = EncodeChoice;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<VariantEncoderComponent, PathCons<Choice, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    EncodeChoice: IsProviderFor<
        PathCons<VariantEncoderComponent, PathCons<Choice, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
impl<
    __Wildcard__,
> DelegateComponent<PathCons<VariantEncoderComponent, PathCons<Nil, __Wildcard__>>>
for App {
    type Delegate = EncodeAny;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<VariantEncoderComponent, PathCons<Nil, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    EncodeAny: IsProviderFor<
        PathCons<VariantEncoderComponent, PathCons<Nil, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
impl<
    __Wildcard__,
> DelegateComponent<
    PathCons<
        VariantEncoderComponent,
        PathCons<
            Struct! { w: f64 },
            __Wildcard__,
        >,
    >,
> for App {
    type Delegate = EncodeAny;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<
        VariantEncoderComponent,
        PathCons<
            Struct! { w: f64 },
            __Wildcard__,
        >,
    >,
    __Context__,
    __Params__,
> for App
where
    EncodeAny: IsProviderFor<
        PathCons<
            VariantEncoderComponent,
            PathCons<
                Struct! { w: f64 },
                __Wildcard__,
            >,
        >,
        __Context__,
        __Params__,
    >,
{}
impl<
    __Wildcard__,
> DelegateComponent<
    PathCons<VariantEncoderComponent, PathCons<Struct!(u8, u16), __Wildcard__>>,
> for App {
    type Delegate = EncodeAny;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<VariantEncoderComponent, PathCons<Struct!(u8, u16), __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    EncodeAny: IsProviderFor<
        PathCons<VariantEncoderComponent, PathCons<Struct!(u8, u16), __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<VariantEncoderComponent, Choice> for App {}
fn main() {}
