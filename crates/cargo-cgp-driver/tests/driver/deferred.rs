//! Tests for the processing of the diagnostics queued during `analysis`
//! ([`cargo_cgp_driver::emitter::drain_queue`]), over plain items standing in for diagnostics.

use std::any::Any;
use std::cell::{Cell, RefCell};
use std::panic;

use cargo_cgp_driver::emitter::{Handling, contained_lookup, drain_queue, unless_fatal};
use rustc_data_structures::FatalErrorMarker;

/// A queue the items handed to `handle` can add to, as a query a transform forces adds a
/// diagnostic, with a record of each item handed on and how.
struct Run {
    queue: RefCell<Vec<u32>>,
    handled: RefCell<Vec<(u32, Handling)>>,
}

impl Run {
    fn with(items: &[u32]) -> Self {
        Run {
            queue: RefCell::new(items.to_vec()),
            handled: RefCell::new(Vec::new()),
        }
    }

    fn take(&self) -> Vec<u32> {
        std::mem::take(&mut *self.queue.borrow_mut())
    }

    fn handled(&self) -> Vec<(u32, Handling)> {
        self.handled.borrow().clone()
    }
}

use Handling::{KeepUntransformed as Keep, Transform};

// Property: the postcondition of `drain_queue`, for a queue nothing adds to: every item is
//   transformed once, in arrival order, and no payload is returned.
#[test]
fn transforms_each_queued_item_in_arrival_order() {
    let run = Run::with(&[3, 1, 2]);
    let stopped = drain_queue(
        || run.take(),
        8,
        || false,
        |item, handling| run.handled.borrow_mut().push((item, handling)),
    );
    assert!(stopped.is_none());
    assert_eq!(
        run.handled(),
        [(3, Transform), (1, Transform), (2, Transform)]
    );
}

// Property: the postcondition of `drain_queue`, for an empty queue: nothing is handed on.
#[test]
fn hands_on_nothing_for_an_empty_queue() {
    let run = Run::with(&[]);
    let stopped = drain_queue(
        || run.take(),
        8,
        || false,
        |item, handling| run.handled.borrow_mut().push((item, handling)),
    );
    assert!(stopped.is_none());
    assert_eq!(run.handled(), []);
}

// Property: the postcondition of `drain_queue`, for items queued while transforming, as a forced
//   query's diagnostic is: they form the next batch and are transformed after the batch that
//   queued them.
#[test]
fn transforms_what_a_transform_queues_in_the_next_batch() {
    let run = Run::with(&[1, 2]);
    let stopped = drain_queue(
        || run.take(),
        8,
        || false,
        |item, handling| {
            run.handled.borrow_mut().push((item, handling));
            if item < 10 {
                run.queue.borrow_mut().push(item + 10);
            }
        },
    );
    assert!(stopped.is_none());
    assert_eq!(
        run.handled(),
        [
            (1, Transform),
            (2, Transform),
            (11, Transform),
            (12, Transform)
        ]
    );
}

// Concern: a transform whose queries emit on every call would loop forever.
// Handled: past `max_rounds` batches, the rest is kept untransformed, and keeping queues nothing.
// Property: the postcondition of `drain_queue`, at the cap: the first `max_rounds` batches are
//   transformed and every later item is kept untransformed.
#[test]
fn keeps_items_past_the_round_cap_untransformed() {
    let run = Run::with(&[0]);
    let stopped = drain_queue(
        || run.take(),
        2,
        || false,
        |item, handling| {
            run.handled.borrow_mut().push((item, handling));
            if handling == Transform {
                run.queue.borrow_mut().push(item + 1);
            }
        },
    );
    assert!(stopped.is_none());
    assert_eq!(run.handled(), [(0, Transform), (1, Transform), (2, Keep)]);
}

// Concern: a fatal error from a query a transform forces unwinds out of the transform, which
//   would lose that diagnostic and every one after it.
// Handled: the unwind is caught; that item, the rest of its batch, and later batches are kept
//   untransformed, and the payload is returned for the caller to resume.
// Property: the postcondition of `drain_queue`, for an unwind out of a transform.
#[test]
fn keeps_the_rest_untransformed_after_an_unwind() {
    let run = Run::with(&[1, 2, 3]);
    let stopped = drain_queue(
        || run.take(),
        8,
        || false,
        |item, handling| {
            if item == 2 && handling == Transform {
                // A queued diagnostic the next batch would hold, then the unwind. `resume_unwind`
                // skips the panic hook, as rustc's `FatalErrorMarker` does.
                run.queue.borrow_mut().push(4);
                panic::resume_unwind(Box::new("fatal"));
            }
            run.handled.borrow_mut().push((item, handling));
        },
    );
    let payload = stopped.expect("the unwind's payload is returned");
    assert_eq!(payload.downcast_ref::<&str>(), Some(&"fatal"));
    assert_eq!(
        run.handled(),
        [(1, Transform), (2, Keep), (3, Keep), (4, Keep)]
    );
}

// Concern: a fatal error kept outside the transforms, by the check-entry lookup or the read of the
//   first batch, left the loop transforming against a compiler that had already raised it.
// Handled: `stop` holding before an item keeps that item, the rest of its batch, and later batches
//   untransformed, and the loop returns no payload, since the caller holds the fatal error.
// Property: the postcondition of `drain_queue`, for `stop` turning true after the first item.
#[test]
fn keeps_the_rest_untransformed_once_stop_holds() {
    let run = Run::with(&[1, 2, 3]);
    let stopped_by_fatal = Cell::new(false);
    let stopped = drain_queue(
        || run.take(),
        8,
        || stopped_by_fatal.get(),
        |item, handling| {
            run.handled.borrow_mut().push((item, handling));
            if item == 1 {
                run.queue.borrow_mut().push(4);
                stopped_by_fatal.set(true);
            }
        },
    );
    assert!(stopped.is_none(), "the caller holds the fatal error");
    assert_eq!(
        run.handled(),
        [(1, Transform), (2, Keep), (3, Keep), (4, Keep)]
    );
}

// Property: the postcondition of `drain_queue`, for `stop` holding from the start, as after a
//   fatal error in the first batch's lookup: every item is kept untransformed.
#[test]
fn keeps_every_item_untransformed_when_stop_holds_at_once() {
    let run = Run::with(&[1, 2]);
    let stopped = drain_queue(
        || run.take(),
        8,
        || true,
        |item, handling| run.handled.borrow_mut().push((item, handling)),
    );
    assert!(stopped.is_none());
    assert_eq!(run.handled(), [(1, Keep), (2, Keep)]);
}

/// The fatal error a lookup kept, if any.
type Fatal = RefCell<Option<Box<dyn Any + Send>>>;

// Property: the postcondition of `contained_lookup`, for a lookup that returns: its answer.
#[test]
fn a_lookup_that_returns_gives_its_answer() {
    let fatal: Fatal = RefCell::new(None);
    assert!(contained_lookup(|| true, &fatal));
    assert!(!contained_lookup(|| false, &fatal));
    assert!(fatal.borrow().is_none());
}

// Concern: the check-entry lookup forces queries outside a transform's containment, so a panic in
//   one would unwind out of the `analysis` wrapper with the batch already taken from the queue.
// Handled: `contained_lookup` contains the panic and answers no.
#[test]
fn a_lookup_that_panics_answers_no() {
    let fatal: Fatal = RefCell::new(None);
    assert!(!contained_lookup(|| panic!("a query bug"), &fatal));
    assert!(fatal.borrow().is_none(), "a panic is not kept as fatal");
}

// Concern: a query the lookup forces can raise a fatal error, which must reach rustc in the end,
//   but not before the diagnostics already taken are buffered.
// Handled: `contained_lookup` keeps the fatal error, answers no, and runs no later lookup.
#[test]
fn a_fatal_error_is_kept_and_stops_later_lookups() {
    let fatal: Fatal = RefCell::new(None);
    let raised = contained_lookup(|| panic::resume_unwind(Box::new(FatalErrorMarker)), &fatal);
    assert!(!raised);
    assert!(
        fatal
            .borrow()
            .as_ref()
            .is_some_and(|p| p.is::<FatalErrorMarker>())
    );
    let ran = RefCell::new(false);
    assert!(!contained_lookup(
        || {
            *ran.borrow_mut() = true;
            true
        },
        &fatal
    ));
    assert!(!*ran.borrow(), "no lookup runs after a fatal error");
}

// Concern: the read of the first batch forces queries under a containment that lets a fatal error
//   through, which would unwind out of the `analysis` wrapper with the batch already taken.
// Handled: `unless_fatal` keeps the fatal error and returns `None`, and runs no later step.
// Property: the postcondition of `unless_fatal`, for a step that returns and for one that raises a
//   fatal error.
#[test]
fn unless_fatal_runs_a_step_and_keeps_its_fatal_error() {
    let fatal: Fatal = RefCell::new(None);
    assert_eq!(unless_fatal(|| 7, &fatal), Some(7));
    assert_eq!(
        unless_fatal(|| panic::resume_unwind(Box::new(FatalErrorMarker)), &fatal),
        None::<()>
    );
    assert!(
        fatal
            .borrow()
            .as_ref()
            .is_some_and(|p| p.is::<FatalErrorMarker>())
    );
    let ran = Cell::new(false);
    assert_eq!(unless_fatal(|| ran.set(true), &fatal), None);
    assert!(!ran.get(), "no step runs after a fatal error");
}
