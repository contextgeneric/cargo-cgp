//! Acceptable failure: a wiring gap surfaced inside a *hand-written* `impl Trait for Context`
//! block — where the caret sits on the impl, never on the context's own type definition. The
//! impl-site anchor recovers the context from the enclosing impl's `Self` type and the *exact*
//! failing obligation (with its concrete component parameter, `CanCalculateArea<Rectangle>`) from
//! the impl's CGP consumer supertrait, then walks it as a check entry would — but heads the tree
//! with the impl's *own* trait, the wrapper the programmer wrote, so the failure reads at their
//! code and descends from there.
//!
//! This is the shape the `cgp-examples/transfer` walkthrough hits with its per-endpoint
//! `impl CanHandleApiSend<Api> for MockApp` blocks: a wrapper trait carrying a generic CGP
//! consumer trait as a supertrait, implemented directly on the context to add a bound (there,
//! `Send`) the component cannot express. Here `App` wires `AreaCalculatorComponent` for
//! `Rectangle` through `ScaleArea`, which depends on the unwired `scale_factor` field; the
//! unsatisfied `App: CanCalculateArea<Rectangle>` supertrait then fails *inside* the
//! `impl CanCalculateAreaChecked<Rectangle> for App` block, at the impl header (`E0277`) and the
//! forwarding `self.area(..)` call (`E0599`). Both recover the same cause and collapse to one
//! block, headed `[CGP-E009] the trait \`CanCalculateAreaChecked<Rectangle>\`` — a wrapper is a
//! plain trait, not a CGP consumer, so it reads "the trait", not "the consumer trait" — over a
//! tree that leads with `CanCalculateAreaChecked`, then its `CanCalculateArea` supertrait, down to
//! the missing field.
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
