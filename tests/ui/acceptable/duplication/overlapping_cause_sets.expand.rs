#![feature(prelude_import)]
//! Failures whose root-cause *sets* overlap without being equal, coalesced into one block.
//!
//! One omitted wiring entry — a single generic key covering every element type — is reached at two
//! instantiations, so it surfaces as two distinct missing-delegate root causes. Each
//! `check_components!` entry reaches one of the two; the use-site call walks every wired component
//! and so reaches both. No two of those three cause sets are equal, so grouping failures by an
//! *identical* cause set left one mistake reported as three blocks — and the use-site block fared
//! worst, because its two top-level roots were exactly the consumers the check blocks had already
//! drawn, so its chain was fully elided against them and it degenerated to a bare `root causes:`
//! list with no dependency chain at all, on the one diagnostic sitting on code the programmer wrote.
//!
//! Pins the shared-cause grouping that replaced it: the three failures form one group because each
//! shares a cause with the union block, so a single `[CGP-E001]` block names both affected consumers,
//! carets all three sites, lists both root causes, and renders both chains in full.
//!
//! Reproduces the [check-trait failure] class.
//!
//! [check-trait failure]: https://github.com/contextgeneric/cgp/blob/main/docs/errors/checks/check-trait-failure.md
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::fmt::Display;
use cgp::prelude::*;
pub trait CanEncode<Value> {
    fn encode(&self, value: &Value) -> String;
}
impl<__Context__, Value> CanEncode<Value> for __Context__
where
    __Context__: Encoder<__Context__, Value>,
{
    fn encode(&self, value: &Value) -> String {
        __Context__::encode(self, value)
    }
}
pub trait Encoder<
    __Context__,
    Value,
>: IsProviderFor<EncoderComponent, __Context__, (Value)> {
    fn encode(__context__: &__Context__, value: &Value) -> String;
}
impl<__Provider__, __Context__, Value> Encoder<__Context__, Value> for __Provider__
where
    __Provider__: DelegateComponent<EncoderComponent>
        + IsProviderFor<EncoderComponent, __Context__, (Value)>,
    <__Provider__ as DelegateComponent<
        EncoderComponent,
    >>::Delegate: Encoder<__Context__, Value>,
{
    fn encode(__context__: &__Context__, value: &Value) -> String {
        <__Provider__ as DelegateComponent<
            EncoderComponent,
        >>::Delegate::encode(__context__, value)
    }
}
pub struct EncoderComponent;
impl<__Context__, Value> Encoder<__Context__, Value> for UseContext
where
    __Context__: CanEncode<Value>,
{
    fn encode(__context__: &__Context__, value: &Value) -> String {
        __Context__::encode(__context__, value)
    }
}
impl<__Context__, Value> IsProviderFor<EncoderComponent, __Context__, (Value)>
for UseContext
where
    __Context__: CanEncode<Value>,
{}
impl<__Context__, Value, __Components__, __Path__> Encoder<__Context__, Value>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Value)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Value)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Value)>>::Output,
    >>::Delegate: Encoder<__Context__, Value>,
{
    fn encode(__context__: &__Context__, value: &Value) -> String {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@Value)>>::Output,
        >>::Delegate::encode(__context__, value)
    }
}
impl<
    __Context__,
    Value,
    __Components__,
    __Path__,
> IsProviderFor<EncoderComponent, __Context__, (Value)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Value)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Value)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Value)>>::Output,
    >>::Delegate: IsProviderFor<EncoderComponent, __Context__, (Value)>
        + Encoder<__Context__, Value>,
{}
/// Encodes any `Display` value directly.
impl<__Context__, Value: Display> Encoder<__Context__, Value> for EncodeDisplay {
    fn encode(__context__: &__Context__, value: &Value) -> String {
        value.to_string()
    }
}
impl<__Context__, Value: Display> IsProviderFor<EncoderComponent, __Context__, (Value)>
for EncodeDisplay {}
pub struct EncodeDisplay;
/// The wrapper a container encoder hands each of its elements to. One generic wiring entry covers
/// every element type at once, so omitting that entry leaves one missing key per element type the
/// wiring actually reaches.
pub struct Element<Value>(pub Value);
pub trait CanReportFirst {
    fn report_first(&self) -> String;
}
impl<__Context__> CanReportFirst for __Context__
where
    __Context__: FirstReporter<__Context__>,
{
    fn report_first(&self) -> String {
        __Context__::report_first(self)
    }
}
pub trait FirstReporter<
    __Context__,
>: IsProviderFor<FirstReporterComponent, __Context__, ()> {
    fn report_first(__context__: &__Context__) -> String;
}
impl<__Provider__, __Context__> FirstReporter<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<FirstReporterComponent>
        + IsProviderFor<FirstReporterComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        FirstReporterComponent,
    >>::Delegate: FirstReporter<__Context__>,
{
    fn report_first(__context__: &__Context__) -> String {
        <__Provider__ as DelegateComponent<
            FirstReporterComponent,
        >>::Delegate::report_first(__context__)
    }
}
pub struct FirstReporterComponent;
impl<__Context__> FirstReporter<__Context__> for UseContext
where
    __Context__: CanReportFirst,
{
    fn report_first(__context__: &__Context__) -> String {
        __Context__::report_first(__context__)
    }
}
impl<__Context__> IsProviderFor<FirstReporterComponent, __Context__, ()> for UseContext
where
    __Context__: CanReportFirst,
{}
impl<__Context__, __Components__, __Path__> FirstReporter<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: FirstReporter<__Context__>,
{
    fn report_first(__context__: &__Context__) -> String {
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate::report_first(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<FirstReporterComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<FirstReporterComponent, __Context__, ()>
        + FirstReporter<__Context__>,
{}
impl<__Context__> FirstReporter<__Context__> for ReportFirst
where
    __Context__: CanEncode<Element<u32>>,
{
    fn report_first(__context__: &__Context__) -> String {
        __context__.encode(&Element(1u32))
    }
}
impl<__Context__> IsProviderFor<FirstReporterComponent, __Context__, ()> for ReportFirst
where
    __Context__: CanEncode<Element<u32>>,
{}
pub struct ReportFirst;
pub trait CanReportSecond {
    fn report_second(&self) -> String;
}
impl<__Context__> CanReportSecond for __Context__
where
    __Context__: SecondReporter<__Context__>,
{
    fn report_second(&self) -> String {
        __Context__::report_second(self)
    }
}
pub trait SecondReporter<
    __Context__,
>: IsProviderFor<SecondReporterComponent, __Context__, ()> {
    fn report_second(__context__: &__Context__) -> String;
}
impl<__Provider__, __Context__> SecondReporter<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<SecondReporterComponent>
        + IsProviderFor<SecondReporterComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        SecondReporterComponent,
    >>::Delegate: SecondReporter<__Context__>,
{
    fn report_second(__context__: &__Context__) -> String {
        <__Provider__ as DelegateComponent<
            SecondReporterComponent,
        >>::Delegate::report_second(__context__)
    }
}
pub struct SecondReporterComponent;
impl<__Context__> SecondReporter<__Context__> for UseContext
where
    __Context__: CanReportSecond,
{
    fn report_second(__context__: &__Context__) -> String {
        __Context__::report_second(__context__)
    }
}
impl<__Context__> IsProviderFor<SecondReporterComponent, __Context__, ()> for UseContext
where
    __Context__: CanReportSecond,
{}
impl<__Context__, __Components__, __Path__> SecondReporter<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: SecondReporter<__Context__>,
{
    fn report_second(__context__: &__Context__) -> String {
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate::report_second(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<SecondReporterComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<SecondReporterComponent, __Context__, ()>
        + SecondReporter<__Context__>,
{}
impl<__Context__> SecondReporter<__Context__> for ReportSecond
where
    __Context__: CanEncode<Element<u64>>,
{
    fn report_second(__context__: &__Context__) -> String {
        __context__.encode(&Element(2u64))
    }
}
impl<__Context__> IsProviderFor<SecondReporterComponent, __Context__, ()>
for ReportSecond
where
    __Context__: CanEncode<Element<u64>>,
{}
pub struct ReportSecond;
pub struct App;
impl DelegateComponent<EncoderComponent> for App {
    type Delegate = RedirectLookup<App, Path!(@EncoderComponent)>;
}
impl<__Context__, __Params__> IsProviderFor<EncoderComponent, __Context__, __Params__>
for App
where
    RedirectLookup<
        App,
        Path!(@EncoderComponent),
    >: IsProviderFor<EncoderComponent, __Context__, __Params__>,
{}
impl DelegateComponent<FirstReporterComponent> for App {
    type Delegate = ReportFirst;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<FirstReporterComponent, __Context__, __Params__> for App
where
    ReportFirst: IsProviderFor<FirstReporterComponent, __Context__, __Params__>,
{}
impl DelegateComponent<SecondReporterComponent> for App {
    type Delegate = ReportSecond;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<SecondReporterComponent, __Context__, __Params__> for App
where
    ReportSecond: IsProviderFor<SecondReporterComponent, __Context__, __Params__>,
{}
impl<
    __Wildcard__,
> DelegateComponent<PathCons<EncoderComponent, PathCons<u32, __Wildcard__>>> for App {
    type Delegate = EncodeDisplay;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__,
> IsProviderFor<
    PathCons<EncoderComponent, PathCons<u32, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    EncodeDisplay: IsProviderFor<
        PathCons<EncoderComponent, PathCons<u32, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
impl<
    __Wildcard__,
> DelegateComponent<PathCons<EncoderComponent, PathCons<u64, __Wildcard__>>> for App {
    type Delegate = EncodeDisplay;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__,
> IsProviderFor<
    PathCons<EncoderComponent, PathCons<u64, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    EncodeDisplay: IsProviderFor<
        PathCons<EncoderComponent, PathCons<u64, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<FirstReporterComponent, ()> for App {}
impl __CheckApp<SecondReporterComponent, ()> for App {}
fn main() {
    let app = App;
    let _ = app.report_first();
}
