//! Data operations on types that lack a data trait no derive can give them.
//!
//! `to_extractor` on the struct `Person` needs `HasExtractor`, whose derive `ExtractField` takes
//! an enum; `builder` on the enum `Shape` needs `HasBuilder`, whose derive `BuildField` takes a
//! struct; and `upcast` from `Option<u64>` needs `HasFields` and `HasExtractor` of a type the crate
//! does not define. Each block states the missing trait, and none names a derive to add.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: CGP-E050 CGP-E104 CGP-E123

use cgp::core::field::impls::CanUpcast;
use cgp::prelude::*;

#[derive(HasField)]
pub struct Person {
    pub name: String,
}

#[derive(HasFields)]
pub enum Shape {
    Circle(f64),
}

#[derive(CgpData)]
pub enum Big {
    Foo(u64),
}

pub fn extract_person(person: Person) {
    let _ = person.to_extractor();
}

pub fn build_shape() {
    let _ = Shape::builder();
}

pub fn upcast_option() {
    let value: Option<u64> = None;
    let _: Big = value.upcast(PhantomData::<Big>);
}

fn main() {}
