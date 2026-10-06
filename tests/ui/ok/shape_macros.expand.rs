#![feature(prelude_import)]
//! Ok: CGP's `Struct!` and `Enum!` shape macros written by hand compile cleanly, and the expansion
//! snapshot shows the expand pass printing every field list in the same form.
//!
//! `Person` and `Shape` derive `HasFields`, whose `Fields` the expansion prints as
//! `Struct! { name: String, age: u8 }` and as an `Enum!` with each variant in its shortest
//! spelling. `rebuild` names the record shape in its bound, and `App` keys an `open` dispatch entry
//! by a shape, so the snapshot also shows a written shape surviving expansion unchanged.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub struct Person {
    pub name: String,
    pub age: u8,
}
impl HasFields for Person {
    type Fields = Struct! { name: String, age: u8 };
}
impl HasFieldsRef for Person {
    type FieldsRef<'__a> = Struct! { name: &'__a String, age: &'__a u8 }
    where
        Self: '__a;
}
impl FromFields for Person {
    fn from_fields(Cons(name, Cons(age, Nil)): Self::Fields) -> Self {
        Self {
            name: name.value,
            age: age.value,
        }
    }
}
impl ToFields for Person {
    fn to_fields(self) -> Self::Fields {
        Cons(self.name.into(), Cons(self.age.into(), Nil))
    }
}
impl ToFieldsRef for Person {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons((&self.name).into(), Cons((&self.age).into(), Nil))
    }
}
pub struct Pair(pub u8, pub u16);
impl HasFields for Pair {
    type Fields = Struct!(u8, u16);
}
impl HasFieldsRef for Pair {
    type FieldsRef<'__a> = Struct!(&'__a u8, &'__a u16) where Self: '__a;
}
impl FromFields for Pair {
    fn from_fields(Cons(field_1, Cons(field_0, Nil)): Self::Fields) -> Self {
        Self(field_1.value, field_0.value)
    }
}
impl ToFields for Pair {
    fn to_fields(self) -> Self::Fields {
        Cons(self.0.into(), Cons(self.1.into(), Nil))
    }
}
impl ToFieldsRef for Pair {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons((&self.0).into(), Cons((&self.1).into(), Nil))
    }
}
pub enum Shape {
    Empty,
    Rect { width: f64, height: f64 },
    Pair(u8, u16),
    Circle(f64),
}
impl HasFields for Shape {
    type Fields = Enum! { Empty, Rect { width: f64, height: f64 }, Pair(u8, u16), Circle(f64) };
}
impl HasFieldsRef for Shape {
    type FieldsRef<'__a> = Enum! { Empty, Rect { width: &'__a f64, height: &'__a f64 }, Pair(&'__a u8, &'__a u16), Circle(&'__a f64) }
    where
        Self: '__a;
}
impl FromFields for Shape {
    fn from_fields(rest: Self::Fields) -> Self {
        match rest {
            Either::Left(field) => {
                let Nil = field.value;
                Self::Empty
            }
            Either::Right(rest) => {
                match rest {
                    Either::Left(field) => {
                        let Cons(width, Cons(height, Nil)) = field.value;
                        Self::Rect {
                            width: width.value,
                            height: height.value,
                        }
                    }
                    Either::Right(rest) => {
                        match rest {
                            Either::Left(field) => {
                                let Cons(field_1, Cons(field_0, Nil)) = field.value;
                                Self::Pair(field_1.value, field_0.value)
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
            }
        }
    }
}
impl ToFields for Shape {
    fn to_fields(self) -> Self::Fields {
        match self {
            Self::Empty => Either::Left(Nil.into()),
            Self::Rect { width, height } => {
                Either::Right(
                    Either::Left(Cons(width.into(), Cons(height.into(), Nil)).into()),
                )
            }
            Self::Pair(field_0, field_1) => {
                Either::Right(
                    Either::Right(
                        Either::Left(
                            Cons(field_0.into(), Cons(field_1.into(), Nil)).into(),
                        ),
                    ),
                )
            }
            Self::Circle(field) => {
                Either::Right(Either::Right(Either::Right(Either::Left(field.into()))))
            }
        }
    }
}
impl ToFieldsRef for Shape {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        match self {
            Self::Empty => Either::Left(Nil.into()),
            Self::Rect { width, height } => {
                Either::Right(
                    Either::Left(Cons(width.into(), Cons(height.into(), Nil)).into()),
                )
            }
            Self::Pair(field_0, field_1) => {
                Either::Right(
                    Either::Right(
                        Either::Left(
                            Cons(field_0.into(), Cons(field_1.into(), Nil)).into(),
                        ),
                    ),
                )
            }
            Self::Circle(field) => {
                Either::Right(Either::Right(Either::Right(Either::Left(field.into()))))
            }
        }
    }
}
pub fn rebuild<T>(
    fields: Struct! { name: String, age: u8 },
) -> T
where
    T: FromFields<
        Fields = Struct! { name: String, age: u8 },
    >,
{
    T::from_fields(fields)
}
pub trait CanDescribeShape<Shape> {
    fn describe_shape(&self) -> &'static str;
}
impl<__Context__, Shape> CanDescribeShape<Shape> for __Context__
where
    __Context__: ShapeDescriber<__Context__, Shape>,
{
    fn describe_shape(&self) -> &'static str {
        __Context__::describe_shape(self)
    }
}
pub trait ShapeDescriber<
    __Context__,
    Shape,
>: IsProviderFor<ShapeDescriberComponent, __Context__, (Shape)> {
    fn describe_shape(__context__: &__Context__) -> &'static str;
}
impl<__Provider__, __Context__, Shape> ShapeDescriber<__Context__, Shape>
for __Provider__
where
    __Provider__: DelegateComponent<ShapeDescriberComponent>
        + IsProviderFor<ShapeDescriberComponent, __Context__, (Shape)>,
    <__Provider__ as DelegateComponent<
        ShapeDescriberComponent,
    >>::Delegate: ShapeDescriber<__Context__, Shape>,
{
    fn describe_shape(__context__: &__Context__) -> &'static str {
        <__Provider__ as DelegateComponent<
            ShapeDescriberComponent,
        >>::Delegate::describe_shape(__context__)
    }
}
pub struct ShapeDescriberComponent;
impl<__Context__, Shape> ShapeDescriber<__Context__, Shape> for UseContext
where
    __Context__: CanDescribeShape<Shape>,
{
    fn describe_shape(__context__: &__Context__) -> &'static str {
        __Context__::describe_shape(__context__)
    }
}
impl<__Context__, Shape> IsProviderFor<ShapeDescriberComponent, __Context__, (Shape)>
for UseContext
where
    __Context__: CanDescribeShape<Shape>,
{}
impl<__Context__, Shape, __Components__, __Path__> ShapeDescriber<__Context__, Shape>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Shape)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Shape)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Shape)>>::Output,
    >>::Delegate: ShapeDescriber<__Context__, Shape>,
{
    fn describe_shape(__context__: &__Context__) -> &'static str {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@Shape)>>::Output,
        >>::Delegate::describe_shape(__context__)
    }
}
impl<
    __Context__,
    Shape,
    __Components__,
    __Path__,
> IsProviderFor<ShapeDescriberComponent, __Context__, (Shape)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Shape)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Shape)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Shape)>>::Output,
    >>::Delegate: IsProviderFor<ShapeDescriberComponent, __Context__, (Shape)>
        + ShapeDescriber<__Context__, Shape>,
{}
impl<__Context__, Shape> ShapeDescriber<__Context__, Shape> for DescribePoint {
    fn describe_shape(__context__: &__Context__) -> &'static str {
        "point"
    }
}
impl<__Context__, Shape> IsProviderFor<ShapeDescriberComponent, __Context__, (Shape)>
for DescribePoint {}
pub struct DescribePoint;
pub struct App;
impl DelegateComponent<ShapeDescriberComponent> for App {
    type Delegate = RedirectLookup<App, Path!(@ShapeDescriberComponent)>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ShapeDescriberComponent, __Context__, __Params__> for App
where
    RedirectLookup<
        App,
        Path!(@ShapeDescriberComponent),
    >: IsProviderFor<ShapeDescriberComponent, __Context__, __Params__>,
{}
impl<
    __Wildcard__,
> DelegateComponent<
    PathCons<
        ShapeDescriberComponent,
        PathCons<
            Struct! { x: f64, y: f64 },
            __Wildcard__,
        >,
    >,
> for App {
    type Delegate = DescribePoint;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<
        ShapeDescriberComponent,
        PathCons<
            Struct! { x: f64, y: f64 },
            __Wildcard__,
        >,
    >,
    __Context__,
    __Params__,
> for App
where
    DescribePoint: IsProviderFor<
        PathCons<
            ShapeDescriberComponent,
            PathCons<
                Struct! { x: f64, y: f64 },
                __Wildcard__,
            >,
        >,
        __Context__,
        __Params__,
    >,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<
    ShapeDescriberComponent,
    Struct! { x: f64, y: f64 },
> for App {}
fn main() {
    let person: Person = rebuild(Cons("Ada".to_owned().into(), Cons(36u8.into(), Nil)));
    {
        match (&person.age, &36) {
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
