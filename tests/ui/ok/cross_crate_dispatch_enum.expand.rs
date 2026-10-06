#![feature(prelude_import)]
//! Dispatch traits declared in one crate, implemented and dispatched in another.
//!
//! `cgp-test-dispatch-traits` declares the `#[cgp_auto_dispatch]` traits and
//! nothing implementing them. This crate defines the payloads, implements the
//! foreign traits for them alongside the foreign enum-level blanket impl, derives
//! the enum, and calls every dispatched method on it. It also runs the foreign
//! per-variant provider `ComputeArea` through a matcher by name, and calls the
//! foreign `#[cgp_computer]` and `#[cgp_producer]` providers. The program compiles
//! clean, so the snapshot of cargo-cgp's output is empty.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::{Computer, Producer, TryComputer};
use cgp::prelude::*;
use cgp_test_dispatch_traits::{
    CanDescribe, CheckedHalve, ComputeArea, Double, HasShape, UnitScale,
};
pub struct Triangle {
    pub base: f64,
    pub height: f64,
}
pub struct Hexagon {
    pub side: f64,
}
pub enum Polygon {
    Triangle(Triangle),
    Hexagon(Hexagon),
}
impl HasFields for Polygon {
    type Fields = Enum! { Triangle(Triangle), Hexagon(Hexagon) };
}
impl HasFieldsRef for Polygon {
    type FieldsRef<'__a> = Enum! { Triangle(&'__a Triangle), Hexagon(&'__a Hexagon) }
    where
        Self: '__a;
}
impl FromFields for Polygon {
    fn from_fields(rest: Self::Fields) -> Self {
        match rest {
            Either::Left(field) => {
                let field = field.value;
                Self::Triangle(field)
            }
            Either::Right(rest) => {
                match rest {
                    Either::Left(field) => {
                        let field = field.value;
                        Self::Hexagon(field)
                    }
                    Either::Right(rest) => match rest {}
                }
            }
        }
    }
}
impl ToFields for Polygon {
    fn to_fields(self) -> Self::Fields {
        match self {
            Self::Triangle(field) => Either::Left(field.into()),
            Self::Hexagon(field) => Either::Right(Either::Left(field.into())),
        }
    }
}
impl ToFieldsRef for Polygon {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        match self {
            Self::Triangle(field) => Either::Left(field.into()),
            Self::Hexagon(field) => Either::Right(Either::Left(field.into())),
        }
    }
}
impl FromVariant<Symbol!("Triangle")> for Polygon {
    type Value = Triangle;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Triangle")>,
        value: Self::Value,
    ) -> Self {
        Self::Triangle(value)
    }
}
impl FromVariant<Symbol!("Hexagon")> for Polygon {
    type Value = Hexagon;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Hexagon")>,
        value: Self::Value,
    ) -> Self {
        Self::Hexagon(value)
    }
}
pub enum __PartialPolygon<__F0__: MapType, __F1__: MapType> {
    Triangle(<__F0__ as MapType>::Map<Triangle>),
    Hexagon(<__F1__ as MapType>::Map<Hexagon>),
}
pub enum __PartialRefPolygon<
    '__a__,
    __R__: MapTypeRef,
    __F0__: MapType,
    __F1__: MapType,
> {
    Triangle(<__F0__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, Triangle>>),
    Hexagon(<__F1__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, Hexagon>>),
}
impl<__F0__: MapType, __F1__: MapType> PartialData for __PartialPolygon<__F0__, __F1__> {
    type Target = Polygon;
}
impl<'__a__, __R__: MapTypeRef, __F0__: MapType, __F1__: MapType> PartialData
for __PartialRefPolygon<'__a__, __R__, __F0__, __F1__> {
    type Target = Polygon;
}
impl HasExtractor for Polygon {
    type Extractor = __PartialPolygon<IsPresent, IsPresent>;
    fn to_extractor(self) -> Self::Extractor {
        match self {
            Self::Triangle(value) => __PartialPolygon::Triangle(value),
            Self::Hexagon(value) => __PartialPolygon::Hexagon(value),
        }
    }
    fn from_extractor(extractor: Self::Extractor) -> Self {
        match extractor {
            __PartialPolygon::Triangle(value) => Self::Triangle(value),
            __PartialPolygon::Hexagon(value) => Self::Hexagon(value),
        }
    }
}
impl HasExtractorRef for Polygon {
    type ExtractorRef<'__a__> = __PartialRefPolygon<'__a__, IsRef, IsPresent, IsPresent>
    where
        Self: '__a__;
    fn extractor_ref<'__a__>(&'__a__ self) -> Self::ExtractorRef<'__a__> {
        match self {
            Self::Triangle(value) => __PartialRefPolygon::Triangle(value),
            Self::Hexagon(value) => __PartialRefPolygon::Hexagon(value),
        }
    }
}
impl HasExtractorMut for Polygon {
    type ExtractorMut<'__a__> = __PartialRefPolygon<'__a__, IsMut, IsPresent, IsPresent>
    where
        Self: '__a__;
    fn extractor_mut<'__a__>(&'__a__ mut self) -> Self::ExtractorMut<'__a__> {
        match self {
            Self::Triangle(value) => __PartialRefPolygon::Triangle(value),
            Self::Hexagon(value) => __PartialRefPolygon::Hexagon(value),
        }
    }
}
impl FinalizeExtract for __PartialPolygon<IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<'__a__, __R__: MapTypeRef> FinalizeExtract
for __PartialRefPolygon<'__a__, __R__, IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<__F1__: MapType> ExtractField<Symbol!("Triangle")>
for __PartialPolygon<IsPresent, __F1__> {
    type Value = Triangle;
    type Remainder = __PartialPolygon<IsVoid, __F1__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Triangle")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialPolygon::Triangle(value) => Ok(value),
            __PartialPolygon::Hexagon(value) => Err(__PartialPolygon::Hexagon(value)),
        }
    }
}
impl<__F0__: MapType> ExtractField<Symbol!("Hexagon")>
for __PartialPolygon<__F0__, IsPresent> {
    type Value = Hexagon;
    type Remainder = __PartialPolygon<__F0__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Hexagon")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialPolygon::Triangle(value) => Err(__PartialPolygon::Triangle(value)),
            __PartialPolygon::Hexagon(value) => Ok(value),
        }
    }
}
impl<'__a__, __R__: MapTypeRef, __F1__: MapType> ExtractField<Symbol!("Triangle")>
for __PartialRefPolygon<'__a__, __R__, IsPresent, __F1__> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, Triangle>;
    type Remainder = __PartialRefPolygon<'__a__, __R__, IsVoid, __F1__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Triangle")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialRefPolygon::Triangle(value) => Ok(value),
            __PartialRefPolygon::Hexagon(value) => {
                Err(__PartialRefPolygon::Hexagon(value))
            }
        }
    }
}
impl<'__a__, __R__: MapTypeRef, __F0__: MapType> ExtractField<Symbol!("Hexagon")>
for __PartialRefPolygon<'__a__, __R__, __F0__, IsPresent> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, Hexagon>;
    type Remainder = __PartialRefPolygon<'__a__, __R__, __F0__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Hexagon")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialRefPolygon::Triangle(value) => {
                Err(__PartialRefPolygon::Triangle(value))
            }
            __PartialRefPolygon::Hexagon(value) => Ok(value),
        }
    }
}
impl HasShape for Triangle {
    fn area(&self) -> f64 {
        self.base * self.height / 2.0
    }
    fn scale(&mut self, factor: f64) {
        self.base *= factor;
        self.height *= factor;
    }
    fn into_name(self) -> &'static str {
        "triangle"
    }
    fn label(&self, prefix: &str) -> String {
        ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("{0} triangle", prefix))
        })
    }
}
impl HasShape for Hexagon {
    fn area(&self) -> f64 {
        2.5 * self.side * self.side
    }
    fn scale(&mut self, factor: f64) {
        self.side *= factor;
    }
    fn into_name(self) -> &'static str {
        "hexagon"
    }
    fn label(&self, prefix: &str) -> String {
        ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("{0} hexagon", prefix))
        })
    }
}
impl CanDescribe for Triangle {
    async fn describe(&self) -> String {
        "a triangle".to_owned()
    }
}
impl CanDescribe for Hexagon {
    async fn describe(&self) -> String {
        "a hexagon".to_owned()
    }
}
pub struct App;
impl DelegateComponent<ErrorTypeProviderComponent> for App {
    type Delegate = UseType<String>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ErrorTypeProviderComponent, __Context__, __Params__> for App
where
    UseType<String>: IsProviderFor<ErrorTypeProviderComponent, __Context__, __Params__>,
{}
fn main() {
    let mut polygon = Polygon::Triangle(Triangle { base: 2.0, height: 3.0 });
    polygon.scale(2.0);
    {
        match (&polygon.area(), &12.0) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    {
        match (&polygon.label("big"), &"big triangle") {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    let _ = polygon.describe();
    {
        match (
            &<MatchWithValueHandlersRef<
                ComputeArea,
            > as Computer<(), (), &Polygon>>::compute(&(), PhantomData, &polygon),
            &12.0,
        ) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    {
        match (&polygon.into_name(), &"triangle") {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    let code = PhantomData::<()>;
    {
        match (&Double::compute(&App, code, 2.0), &4.0) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    {
        match (&CheckedHalve::try_compute(&App, code, 3), &Err("3 is odd".to_owned())) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    {
        match (&UnitScale::produce(&App, code), &1.0) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
}
