//! A dispatcher wired as the entry for the type it dispatches, so its variant handlers loop back.
//!
//! `MatchWithValueHandlers` evaluates an `Expr` by handing each variant's payload back to the
//! context through its inner `UseContext`, and `EvalAdd` evaluates a `Plus<Expr>` by asking the
//! context to evaluate each operand, an `Expr` again. Wired as `Interp`'s entry for `Expr`, the
//! dispatcher makes that recursion part of the trait resolution, so checking either key overflows
//! (`E0275`). The fix is a wrapper provider for `Expr` whose impl has no `where` clause and calls
//! the dispatcher in its body, as the expression interpreter example does.
//!
//! CGP error class: wiring cycle (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/wiring-cycle.md).

//@quality: usability
//@codes: CGP-E020

use cgp::extra::dispatch::MatchWithValueHandlers;
use cgp::extra::handler::{CanCompute, ComputerComponent};
use cgp::prelude::*;

pub struct Eval;

pub struct Plus<Expr> {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

pub struct Literal(pub u64);

#[derive(CgpData)]
pub enum Expr {
    Plus(Plus<Expr>),
    Literal(Literal),
}

#[cgp_impl(new EvalAdd)]
#[uses(CanCompute<Code, Expr, Output = u64>)]
impl<Code> Computer<Code, Plus<Expr>> {
    type Output = u64;

    fn compute(&self, code: PhantomData<Code>, Plus { left, right }: Plus<Expr>) -> u64 {
        self.compute(code, *left) + self.compute(code, *right)
    }
}

#[cgp_impl(new EvalLiteral)]
impl<Code> Computer<Code, Literal> {
    type Output = u64;

    fn compute(&self, _code: PhantomData<Code>, Literal(value): Literal) -> u64 {
        value
    }
}

pub struct Interp;

delegate_components! {
    Interp {
        open ComputerComponent;

        @ComputerComponent.<Code> Code.Expr: MatchWithValueHandlers,
        @ComputerComponent.<Code> Code.Plus<Expr>: EvalAdd,
        @ComputerComponent.<Code> Code.Literal: EvalLiteral,
    }
}

check_components! {
    Interp {
        ComputerComponent: [
            (Eval, Expr),
            (Eval, Plus<Expr>),
        ],
    }
}

fn main() {}
