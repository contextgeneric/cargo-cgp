#![feature(prelude_import)]
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
impl<__Context__> VariantEncoder<__Context__, u64> for EncodeU64 {
    fn encode_variant(__context__: &__Context__, value: &u64) -> String {
        value.to_string()
    }
}
impl<__Context__> IsProviderFor<VariantEncoderComponent, __Context__, (u64)>
for EncodeU64 {}
pub struct EncodeU64;
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
    Sum![
        Field<Symbol!("Rect"), u64>, Field<Symbol!("Circle"), f64>
    ]: EncodeVariants<__Context__>,
{
    fn encode_variant(__context__: &__Context__, _value: &Choice) -> String {
        String::new()
    }
}
impl<__Context__> IsProviderFor<VariantEncoderComponent, __Context__, (Choice)>
for EncodeChoice
where
    Sum![
        Field<Symbol!("Rect"), u64>, Field<Symbol!("Circle"), f64>
    ]: EncodeVariants<__Context__>,
{}
pub struct EncodeChoice;
pub struct App;
impl DelegateComponent<VariantEncoderComponent> for App {
    type Delegate = RedirectLookup<App, Path!(@VariantEncoderComponent)>;
}
impl<
    __Context__,
    __Params__,
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
    __Params__,
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
> DelegateComponent<PathCons<VariantEncoderComponent, PathCons<u64, __Wildcard__>>>
for App {
    type Delegate = EncodeU64;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__,
> IsProviderFor<
    PathCons<VariantEncoderComponent, PathCons<u64, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    EncodeU64: IsProviderFor<
        PathCons<VariantEncoderComponent, PathCons<u64, __Wildcard__>>,
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
