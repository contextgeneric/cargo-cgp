//! Scheduling fixtures across a pool of workers so they run in parallel.
//!
//! Each worker owns one throwaway crate (see [`crate::harness`]), and fixtures are pulled
//! from a shared cursor so a fast worker keeps taking work rather than waiting on a slow
//! one. The catch cargo forces on us is that a `cargo` build holds an exclusive lock on
//! its target directory for the whole build, so two workers can only run at once by
//! building in *separate* target directories — which is exactly what per-worker crates
//! give us, at the cost of compiling cgp once per worker.
//!
//! Each fixture's result is reported the moment it finishes, through a `done` callback the
//! runner calls one at a time (never overlapping), so a run streams live rather than
//! withholding everything to the end. The order is completion order, not fixture order —
//! whichever worker finishes first reports first — so a caller that needs its output
//! identifiable prints the fixture's name on every line.

use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

/// The default worker count when `--jobs` is not given: the machine's parallelism, but
/// never more workers than there are fixtures (extra workers would only idle and rebuild
/// cgp for nothing). Always at least one.
pub fn default_jobs(fixture_count: usize) -> usize {
    thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(fixture_count)
        .max(1)
}

/// Run `task` over every fixture across `jobs` workers, calling `done` with each result as
/// soon as it is ready. Worker `w` (in `0..jobs`) is passed to `task` alongside the
/// fixture so the task can address that worker's own crate; the same worker index recurs
/// for every fixture that worker picks up, so its crate stays warm across them. `done` is
/// serialized — the runner never calls it from two workers at once — so it can print
/// without interleaving; the results arrive in completion order.
pub fn run<T, Task, Done>(jobs: usize, fixtures: &[&Path], task: Task, done: Done)
where
    Task: Fn(usize, &Path) -> T + Sync,
    Done: Fn(T) + Sync,
{
    let next = AtomicUsize::new(0);
    let done_lock = Mutex::new(());

    thread::scope(|scope| {
        for worker in 0..jobs {
            let next = &next;
            let task = &task;
            let done = &done;
            let done_lock = &done_lock;
            scope.spawn(move || {
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some(fixture) = fixtures.get(index) else {
                        break;
                    };
                    let result = task(worker, fixture);
                    let _guard = done_lock.lock().unwrap();
                    done(result);
                }
            });
        }
    });
}
