//! Clean compile: an enum whose variants are named after the associated types the variant derives
//! implement, under `#[derive(CgpData)]`.
//!
//! The derives name `Value`, `Remainder`, `Extractor`, `ExtractorRef`, `ExtractorMut`, `Fields`, and
//! `FieldsRef` through qualified projections such as `<Self as ExtractField<Tag>>::Value`, which the
//! expansion shows, so a variant with any of those names cannot make a generated path ambiguous.

use cgp::prelude::*;

#[derive(CgpData)]
pub enum Token {
    Value(u64),
    Remainder(u64),
    Extractor(String),
    ExtractorRef(String),
    ExtractorMut(String),
    Fields(bool),
    FieldsRef(char),
}

fn main() {}
