//! Ok: CGP's `Struct!` and `Enum!` shape macros written by hand compile cleanly, and the expansion
//! snapshot shows the expand pass printing every field list in the same form.
//!
//! `Person` and `Shape` derive `HasFields`, whose `Fields` the expansion prints as
//! `Struct! { name: String, age: u8 }` and as an `Enum!` with each variant in its shortest
//! spelling. `rebuild` names the record shape in its bound, and `App` keys an `open` dispatch entry
//! by a shape, so the snapshot also shows a written shape surviving expansion unchanged.

use cgp::prelude::*;

#[derive(HasFields)]
pub struct Person {
    pub name: String,
    pub age: u8,
}

#[derive(HasFields)]
pub struct Pair(pub u8, pub u16);

#[derive(HasFields)]
pub enum Shape {
    Empty,
    Rect { width: f64, height: f64 },
    Pair(u8, u16),
    Circle(f64),
}

pub fn rebuild<T>(fields: Struct! { name: String, age: u8 }) -> T
where
    T: FromFields<Fields = Struct! { name: String, age: u8 }>,
{
    T::from_fields(fields)
}

#[cgp_component(ShapeDescriber)]
pub trait CanDescribeShape<Shape> {
    fn describe_shape(&self) -> &'static str;
}

#[cgp_impl(new DescribePoint)]
impl<Shape> ShapeDescriber<Shape> {
    fn describe_shape(&self) -> &'static str {
        "point"
    }
}

pub struct App;

delegate_components! {
    App {
        open ShapeDescriberComponent;

        @ShapeDescriberComponent.Struct! { x: f64, y: f64 }: DescribePoint,
    }
}

check_components! {
    App {
        ShapeDescriberComponent: Struct! { x: f64, y: f64 },
    }
}

fn main() {
    let person: Person = rebuild(product!["Ada".to_owned().into(), 36u8.into()]);
    assert_eq!(person.age, 36);
}
