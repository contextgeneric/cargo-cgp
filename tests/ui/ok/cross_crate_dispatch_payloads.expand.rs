#![feature(prelude_import)]
//! Dispatch traits, payloads, and enums spread over three crates.
//!
//! `cgp-test-dispatch-traits` declares the `#[cgp_auto_dispatch]` traits,
//! `cgp-test-dispatch-shapes` implements them for its payloads `Circle` and
//! `Square` and derives an enum `Shape` over them, and this crate both calls the
//! dispatched methods on that foreign enum and derives its own enum over the same
//! foreign payloads, which gains the traits through the blanket impl declared two
//! crates up. The program compiles clean, so the snapshot of cargo-cgp's output is
//! empty.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
use cgp_test_dispatch_shapes::{Circle, Shape, Square};
use cgp_test_dispatch_traits::{CanDescribe, HasShape};
pub enum Tile {
    Square(Square),
    Circle(Circle),
}
impl HasFields for Tile {
    type Fields = Sum![
        Field<Symbol!("Square"), Square>, Field<Symbol!("Circle"), Circle>
    ];
}
impl HasFieldsRef for Tile {
    type FieldsRef<'__a> = Sum![
        Field<Symbol!("Square"), &'__a Square>, Field<Symbol!("Circle"), &'__a
        Circle>
    ]
    where
        Self: '__a;
}
impl FromFields for Tile {
    fn from_fields(rest: Self::Fields) -> Self {
        match rest {
            Either::Left(field) => {
                let field = field.value;
                Self::Square(field)
            }
            Either::Right(rest) => {
                match rest {
                    Either::Left(field) => {
                        let field = field.value;
                        Self::Circle(field)
                    }
                    Either::Right(rest) => match rest {}
                }
            }
        }
    }
}
impl ToFields for Tile {
    fn to_fields(self) -> Self::Fields {
        match self {
            Self::Square(field) => Either::Left(field.into()),
            Self::Circle(field) => Either::Right(Either::Left(field.into())),
        }
    }
}
impl ToFieldsRef for Tile {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        match self {
            Self::Square(field) => Either::Left(field.into()),
            Self::Circle(field) => Either::Right(Either::Left(field.into())),
        }
    }
}
impl FromVariant<Symbol!("Square")> for Tile {
    type Value = Square;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Square")>,
        value: Self::Value,
    ) -> Self {
        Self::Square(value)
    }
}
impl FromVariant<Symbol!("Circle")> for Tile {
    type Value = Circle;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Circle")>,
        value: Self::Value,
    ) -> Self {
        Self::Circle(value)
    }
}
pub enum __PartialTile<__F0__: MapType, __F1__: MapType> {
    Square(<__F0__ as MapType>::Map<Square>),
    Circle(<__F1__ as MapType>::Map<Circle>),
}
pub enum __PartialRefTile<'__a__, __R__: MapTypeRef, __F0__: MapType, __F1__: MapType> {
    Square(<__F0__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, Square>>),
    Circle(<__F1__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, Circle>>),
}
impl<__F0__: MapType, __F1__: MapType> PartialData for __PartialTile<__F0__, __F1__> {
    type Target = Tile;
}
impl<'__a__, __R__: MapTypeRef, __F0__: MapType, __F1__: MapType> PartialData
for __PartialRefTile<'__a__, __R__, __F0__, __F1__> {
    type Target = Tile;
}
impl HasExtractor for Tile {
    type Extractor = __PartialTile<IsPresent, IsPresent>;
    fn to_extractor(self) -> Self::Extractor {
        match self {
            Self::Square(value) => __PartialTile::Square(value),
            Self::Circle(value) => __PartialTile::Circle(value),
        }
    }
    fn from_extractor(extractor: Self::Extractor) -> Self {
        match extractor {
            __PartialTile::Square(value) => Self::Square(value),
            __PartialTile::Circle(value) => Self::Circle(value),
        }
    }
}
impl HasExtractorRef for Tile {
    type ExtractorRef<'__a__> = __PartialRefTile<'__a__, IsRef, IsPresent, IsPresent>
    where
        Self: '__a__;
    fn extractor_ref<'__a__>(&'__a__ self) -> Self::ExtractorRef<'__a__> {
        match self {
            Self::Square(value) => __PartialRefTile::Square(value),
            Self::Circle(value) => __PartialRefTile::Circle(value),
        }
    }
}
impl HasExtractorMut for Tile {
    type ExtractorMut<'__a__> = __PartialRefTile<'__a__, IsMut, IsPresent, IsPresent>
    where
        Self: '__a__;
    fn extractor_mut<'__a__>(&'__a__ mut self) -> Self::ExtractorMut<'__a__> {
        match self {
            Self::Square(value) => __PartialRefTile::Square(value),
            Self::Circle(value) => __PartialRefTile::Circle(value),
        }
    }
}
impl FinalizeExtract for __PartialTile<IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<'__a__, __R__: MapTypeRef> FinalizeExtract
for __PartialRefTile<'__a__, __R__, IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<__F1__: MapType> ExtractField<Symbol!("Square")>
for __PartialTile<IsPresent, __F1__> {
    type Value = Square;
    type Remainder = __PartialTile<IsVoid, __F1__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Square")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialTile::Square(value) => Ok(value),
            __PartialTile::Circle(value) => Err(__PartialTile::Circle(value)),
        }
    }
}
impl<__F0__: MapType> ExtractField<Symbol!("Circle")>
for __PartialTile<__F0__, IsPresent> {
    type Value = Circle;
    type Remainder = __PartialTile<__F0__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Circle")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialTile::Square(value) => Err(__PartialTile::Square(value)),
            __PartialTile::Circle(value) => Ok(value),
        }
    }
}
impl<'__a__, __R__: MapTypeRef, __F1__: MapType> ExtractField<Symbol!("Square")>
for __PartialRefTile<'__a__, __R__, IsPresent, __F1__> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, Square>;
    type Remainder = __PartialRefTile<'__a__, __R__, IsVoid, __F1__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Square")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialRefTile::Square(value) => Ok(value),
            __PartialRefTile::Circle(value) => Err(__PartialRefTile::Circle(value)),
        }
    }
}
impl<'__a__, __R__: MapTypeRef, __F0__: MapType> ExtractField<Symbol!("Circle")>
for __PartialRefTile<'__a__, __R__, __F0__, IsPresent> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, Circle>;
    type Remainder = __PartialRefTile<'__a__, __R__, __F0__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Circle")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialRefTile::Square(value) => Err(__PartialRefTile::Square(value)),
            __PartialRefTile::Circle(value) => Ok(value),
        }
    }
}
fn main() {
    let mut shape = Shape::Circle(Circle { radius: 1.0 });
    shape.scale(2.0);
    {
        match (&shape.area(), &12.0) {
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
    let _ = shape.describe();
    {
        match (&shape.into_name(), &"circle") {
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
    let tile = Tile::Square(Square { side: 3.0 });
    {
        match (&tile.area(), &9.0) {
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
        match (&tile.label("a"), &"a square") {
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
        match (&tile.into_name(), &"square") {
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
