//! Tests for the processing of the diagnostics queued during `analysis`
//! ([`cargo_cgp_driver::emitter::drain_queue`]), over plain items standing in for diagnostics.

use std::cell::RefCell;
use std::panic;

use cargo_cgp_driver::emitter::{Handling, drain_queue};

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
