//! # Variables in Depth
//!
//! The book offers two models of what a variable *is*, and insists on both:
//!
//! * The **high-level model** — a variable is a *name for a value*. Values
//!   *flow* from one access to the next, and the borrow checker verifies that
//!   flows which can exist in parallel are compatible. This is roughly how the
//!   compiler and the borrow checker actually reason, and it is the model to
//!   reach for when thinking about lifetimes and borrows.
//! * The **low-level model** — a variable is a *value slot* at some address,
//!   which may or may not currently hold a legal value. This is the C/C++ memory
//!   model, and it is the model to reach for with raw pointers and unsafe code.
//!
//! Neither is complete. Both are useful. The `compile_fail` doctests below put
//! the borrow checker's rejections on the record.

use std::mem::MaybeUninit;
use std::mem::size_of;
use std::mem::size_of_val;
use std::ptr;

// ---------------------------------------------------------------------------
// The high-level model: variables are names, and values flow
// ---------------------------------------------------------------------------

/// Listing 1-2, minus its final line — and therefore legal.
///
/// The shared flow out of `x` ("`&x`") terminates at the `assert_eq!` below it.
/// Once that flow is over, the exclusive flow from `x = 43` has no conflicting
/// flow to overlap with, so the borrow checker is satisfied. The book makes this
/// point explicitly: "if [the last line] was not there, this code would compile
/// fine!"
pub fn shared_flow_ends_before_the_write() {
    let mut x; // NOTE: no value yet, so no flow can be drawn from it
    x = 42; // first flow starts here
    let y = &x; // a second, shared flow from the same value
    assert_eq!(*y, 42); // ...which terminates here
    x = 43; // exclusive flow: no live shared flow to conflict with
    assert_eq!(x, 43);
}

/// The illegal version, for contrast: the shared flow now runs *through* the
/// write, so the exclusive flow at `x = 43` overlaps a live borrow.
///
/// This is Listing 1-2 exactly. It does not compile:
///
/// ```compile_fail,E0506
/// let mut x;
/// x = 42;
/// let y = &x;
/// x = 43;              // cannot assign to `x` because it is borrowed
/// assert_eq!(*y, 42);
/// ```
pub fn listing_1_2_is_rejected() {}

/// Using a variable before it holds a value leaves the borrow checker nowhere to
/// draw a flow *from*, which is why the compiler treats an uninitialized local
/// as effectively non-existent rather than as "some bytes":
///
/// ```compile_fail,E0381
/// let x: i32;
/// assert_eq!(x, 42);
/// ```
///
/// The same reasoning covers moved values: after a move there is no live flow to
/// the old variable, even though its bytes are still sitting there.
///
/// ```compile_fail,E0382
/// let owned = String::from("moved");
/// let new_owner = owned;
/// let n = owned.len();  // use of moved value
/// ```
pub fn no_flow_to_draw_from() {}

/// Shadowing declares a *brand-new* variable. The two coexist — the later one
/// just takes over the name, so subsequent code can no longer name the earlier
/// one. That they may have entirely different types is the simplest proof they
/// are distinct variables, and the low-level model agrees: distinct variables
/// get distinct places.
pub fn shadowing_creates_a_new_variable() {
    let x = 5; // an `i32`
    let outer_addr = ptr::addr_of!(x) as *const ();

    let x = "five"; // a different variable, of a different type
    let inner_addr = ptr::addr_of!(x) as *const ();

    assert_eq!(x.len(), 4);
    // Both variables are still alive here, so their places cannot overlap.
    assert_ne!(outer_addr, inner_addr);
}

// ---------------------------------------------------------------------------
// The low-level model: variables are value slots at a stable address
// ---------------------------------------------------------------------------

/// In the low-level model the slot exists — with an address — before it holds a
/// legal value. `let x: usize;` reserves room for a `usize` on the stack but
/// leaves it empty; `x = 6` fills it.
///
/// Rust is stricter than the model: it will not even hand you the *address* of a
/// declared-but-unassigned local, because until it is assigned the variable does
/// not count as a place at all.
///
/// ```compile_fail,E0381
/// let x: usize;
/// let p = std::ptr::addr_of!(x);  // `x` used here but it is possibly-uninitialized
/// ```
///
/// `MaybeUninit<T>` is the explicit opt-in to the low-level model: a real slot
/// with an address that may or may not hold a legal value. (Actually *reading* it
/// requires unsafe code — previewing Chapter 9 — because the compiler can no
/// longer vouch for the contents.)
pub fn the_slot_exists_before_it_is_filled() {
    let mut slot = MaybeUninit::<usize>::uninit();

    // The place and its address exist now, before any value does.
    let before = slot.as_ptr();

    slot.write(6); // the slot is filled; its address does not change

    let after = slot.as_ptr();
    assert_eq!(before, after);

    // SAFETY: `write` initialized the slot above and nothing has moved the value
    // out since, so the bytes are a valid `usize`.
    let value = unsafe { slot.assume_init() };

    assert_eq!(value, 6);
    assert_eq!(size_of_val(&value), size_of::<usize>());
}

/// Assigning to a variable replaces the *contents* of its slot. The place itself
/// does not move, which is why `&x` stays valid across the assignment — and why
/// a pointer to a variable can be dereferenced to observe the new value.
pub fn assignment_writes_the_slot_in_place() {
    let mut x: usize = 6;
    let addr_before = ptr::addr_of!(x);

    x = 7;

    let addr_after = ptr::addr_of!(x);
    assert_eq!(addr_before, addr_after);

    let slot: &mut usize = &mut x;
    *slot = 8; // write through a pointer: same place, new value
    assert_eq!(x, 8);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn shared_flow_ends_before_the_write_runs() {
        shared_flow_ends_before_the_write();
    }

    #[test]
    fn shadowing_creates_a_new_variable_runs() {
        shadowing_creates_a_new_variable();
    }

    #[test]
    fn the_slot_exists_before_it_is_filled_runs() {
        the_slot_exists_before_it_is_filled();
    }

    #[test]
    fn assignment_writes_the_slot_in_place_runs() {
        assignment_writes_the_slot_in_place();
    }
}
