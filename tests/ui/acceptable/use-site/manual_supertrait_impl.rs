//! Acceptable failure: a wiring gap that surfaces inside a *hand-written* `impl Trait for
//! Context` block — where the caret sits on the impl, never on the context's own type
//! definition — is resolved to the same compact `root cause:` tree a `check_components!` entry
//! gets. This is the impl-site anchor: the resolver recovers the context from the enclosing
//! impl's `Self` type and the *exact* failing obligation (with its concrete component
//! parameter, `CanCalculateArea<Rectangle>`) from the impl's CGP consumer supertrait, then
//! walks it exactly as a check entry would.
//!
//! This is the shape the `cgp-examples/transfer` walkthrough hits with its per-endpoint
//! `impl CanHandleApiSend<Api> for MockApp` blocks: a wrapper trait carrying a generic CGP
//! consumer trait as a supertrait, implemented directly on the context to add a bound (there,
//! `Send`) the component cannot express. Here `App` wires `AreaCalculatorComponent` for
//! `Rectangle` through `ScaleArea`, which depends on the unwired `scale_factor` field; the
//! unsatisfied `App: CanCalculateArea<Rectangle>` supertrait then fails *inside* the
//! `impl CanCalculateAreaChecked<Rectangle> for App` block — reported both at the impl header
//! (`E0277`) and at the forwarding `self.area(..)` call in its body (`E0599`), each over the
//! same `missing field \`scale_factor\`` root-cause tree.
//!
//! See docs/implementation/typed-root-cause-resolution.md (the impl-site path).

use cgp::prelude::*;

pub struct Rectangle;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: PhantomData<Shape>) -> f64;
}

#[cgp_auto_getter]
pub trait HasScaleFactor {
    fn scale_factor(&self) -> &f64;
}

// Depends on a `scale_factor` field the context is never given.
#[cgp_impl(new ScaleArea)]
#[uses(HasScaleFactor)]
impl<Shape> AreaCalculator<Shape> {
    fn area(&self, _shape: PhantomData<Shape>) -> f64 {
        *self.scale_factor()
    }
}

pub struct App;

// Accepted even though `App` has no `scale_factor` field that `ScaleArea` needs.
delegate_components! {
    App {
        open AreaCalculatorComponent;

        @AreaCalculatorComponent.Rectangle: ScaleArea,
    }
}

// A hand-written wrapper trait carrying the generic CGP consumer trait as a supertrait,
// implemented directly on the concrete context — the `CanHandleApiSend` shape from the
// transfer example. The unsatisfied `App: CanCalculateArea<Rectangle>` supertrait surfaces
// here, inside the impl block, with no span on `App`'s own definition.
pub trait CanCalculateAreaChecked<Shape>: CanCalculateArea<Shape> {
    fn area_checked(&self, shape: PhantomData<Shape>) -> f64;
}

impl CanCalculateAreaChecked<Rectangle> for App {
    fn area_checked(&self, shape: PhantomData<Rectangle>) -> f64 {
        self.area(shape)
    }
}

fn main() {}
