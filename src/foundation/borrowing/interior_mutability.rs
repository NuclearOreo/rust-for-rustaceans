//! # Interior Mutability
//!
//! "Some types provide *interior mutability*, meaning they allow you to mutate a
//! value through a shared reference." They can do this because they bring their
//! own guarantee to replace the one `&mut` normally provides — either a runtime
//! check, an atomic instruction, or an invariant about what references they ever
//! hand out.
//!
//! The book splits them into two categories, and the split is about *what they
//! give you access to*:
//!
//! 1. **Types that hand out a `&mut` through a `&`** — `Mutex`, `RefCell`. They
//!    must ensure that "for any value they give a mutable reference to, only one
//!    mutable reference (and no shared references) can exist at a time." All of
//!    them are built on `UnsafeCell`, which "is the *only* correct way to mutate
//!    through a shared reference" (Chapter 9).
//! 2. **Types that never hand out a reference, only in-place operations** — the
//!    atomics in `std::sync::atomic`, and `Cell`. You "cannot get a reference
//!    directly to the `usize` or `i32` behind such a type, but you can read and
//!    replace its value at a given point in time."
//!
//! The distinction matters because it is the source of each type's safety
//! argument: a `Mutex` enforces exclusivity, while a `Cell` sidesteps the problem
//! by ensuring no reference to its contents can ever exist.

use std::cell::Cell;
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::thread;

// ---------------------------------------------------------------------------
// Category 1: a `&mut` obtained through a `&`
// ---------------------------------------------------------------------------

/// `RefCell` is the single-threaded member of the first category. The `&mut` it
/// hands out is negotiated at runtime rather than at compile time.
pub fn refcell_lends_out_a_mutable_reference() {
    let value = RefCell::new(0);
    let shared = &value; // only a shared reference in hand

    *shared.borrow_mut() += 1;
    *shared.borrow_mut() += 1;

    assert_eq!(*shared.borrow(), 2);
}

/// The exclusivity contract still holds — it has just moved from compile time to
/// run time. Asking for a second `&mut` while the first is alive fails instead of
/// producing two aliasing references.
pub fn refcell_enforces_exclusivity_at_runtime() {
    let cell = RefCell::new(0);

    let first = cell.borrow_mut();
    assert!(cell.try_borrow_mut().is_err()); // no second `&mut`
    drop(first);

    assert!(cell.try_borrow_mut().is_ok()); // released
}

/// `Mutex` provides the same guarantee across threads, using an atomic
/// test-and-set rather than a counter. A `Mutex<T>` is `Sync`, so a shared
/// `&Mutex<T>` can be handed to several threads at once.
pub fn mutex_shares_mutation_across_threads() {
    let counter = Arc::new(Mutex::new(0usize));

    let handles: Vec<_> = (0..4)
        .map(|_| {
            let counter = Arc::clone(&counter);
            thread::spawn(move || {
                for _ in 0..100 {
                    *counter.lock().unwrap() += 1;
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(*counter.lock().unwrap(), 400);
}

// ---------------------------------------------------------------------------
// Category 2: in-place manipulation, no references handed out
// ---------------------------------------------------------------------------

/// `Cell` is the interesting one. It "is not shareable across threads and never
/// gives out a reference to the value contained in the `Cell`." Every method
/// either returns a *copy* of the contents or replaces them wholesale.
///
/// Since no reference to the inner value can exist, moving it is always fine; and
/// since `Cell` is not `Sync`, there is no other thread that could be mutating it
/// concurrently.
pub fn cell_manipulates_in_place_without_handing_out_references() {
    let cell = Cell::new(1);

    cell.set(cell.get() + 1); // copy out, add, copy back in
    assert_eq!(cell.get(), 2);

    let previous = cell.replace(10); // swap in a new value, get the old one
    assert_eq!(previous, 2);
    assert_eq!(cell.get(), 10);
}

/// The atomics are the thread-safe version of the same idea: no reference to the
/// inner integer, just operations that are atomic with respect to other threads.
pub fn atomics_mutate_in_place_across_threads() {
    let counter = AtomicUsize::new(0);

    counter.fetch_add(1, Ordering::SeqCst);

    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn refcell_lends_out_a_mutable_reference_runs() {
        refcell_lends_out_a_mutable_reference();
    }

    #[test]
    fn refcell_enforces_exclusivity_at_runtime_runs() {
        refcell_enforces_exclusivity_at_runtime();
    }

    #[test]
    fn mutex_shares_mutation_across_threads_runs() {
        mutex_shares_mutation_across_threads();
    }

    #[test]
    fn cell_manipulates_in_place_without_handing_out_references_runs() {
        cell_manipulates_in_place_without_handing_out_references();
    }

    #[test]
    fn atomics_mutate_in_place_across_threads_runs() {
        atomics_mutate_in_place_across_threads();
    }
}
