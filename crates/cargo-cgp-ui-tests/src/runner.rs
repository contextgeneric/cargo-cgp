//! Scheduling fixtures across a pool of workers so they run in parallel.
//!
//! Each worker owns one throwaway crate (see [`crate::harness`]), and fixtures are pulled
//! from a shared cursor so a fast worker keeps taking work rather than waiting on a slow
//! one. The catch cargo forces on us is that a `cargo` build holds an exclusive lock on
//! its target directory for the whole build, so two workers can only run at once by
//! building in *separate* target directories — which is exactly what per-worker crates
//! give us, at the cost of compiling cgp once per worker.
//!
//! Output is kept in fixture order regardless of finish order: each worker stores its
//! result in the slot for the fixture's index, and [`run`] returns the slots in order for
//! the caller to print. Nothing is printed here.

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

/// Run `task` over every fixture across `jobs` workers, returning one result per fixture
/// in the original order. Worker `w` (in `0..jobs`) is passed to `task` alongside the
/// fixture so the task can address that worker's own crate; the same worker index recurs
/// for every fixture that worker picks up, so its crate stays warm across them.
pub fn run<T, F>(jobs: usize, fixtures: &[&Path], task: F) -> Vec<T>
where
    T: Send,
    F: Fn(usize, &Path) -> T + Sync,
{
    let next = AtomicUsize::new(0);
    let slots: Vec<Mutex<Option<T>>> = (0..fixtures.len()).map(|_| Mutex::new(None)).collect();

    thread::scope(|scope| {
        for worker in 0..jobs {
            let next = &next;
            let slots = &slots;
            let task = &task;
            scope.spawn(move || {
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some(fixture) = fixtures.get(index) else {
                        break;
                    };
                    let result = task(worker, fixture);
                    *slots[index].lock().unwrap() = Some(result);
                }
            });
        }
    });

    slots
        .into_iter()
        .map(|slot| slot.into_inner().unwrap().expect("every fixture was run"))
        .collect()
}
