//! # Mutable References
//!
//! "The alternative to a shared reference is a mutable reference: `&mut T`." Its
//! contract is *exclusivity*: "the compiler assumes that there are no other
//! threads accessing the target value, whether through a shared reference or a
//! mutable one."
//!
//! Exclusivity is what makes ordinary optimisations safe. Two `&mut` cannot
//! alias, so a write through one cannot be observed through the other, so loads
//! can be reordered and hoisted. The compiler does not have to prove the absence
//! of aliasing — the type system already did.
//!
//! Three listings follow: the no-aliasing assumption (1-5), the fact that
//! mutability reaches only the immediately referenced memory (1-6), and the rule
//! that moving a value out through `&mut` must leave a legal value behind (1-7).

use std::sync::Arc;

/// Listing 1-5: *Rust assumes that mutable references are exclusive.*
///
/// Because `input` and `output` cannot overlap, the reassignment of `*output`
/// cannot affect the second test of `*input`, so this is just an `if`/`else`.
///
/// The optimisation is only sound because `noalias(&x, &mut x)` is rejected —
/// that call would let a `1` become a `3`:
///
/// ```compile_fail,E0502
/// fn noalias(input: &i32, output: &mut i32) {
///     if *input == 1 { *output = 2; }
///     if *input != 1 { *output = 3; }
/// }
///
/// let mut x = 1;
/// noalias(&x, &mut x);   // cannot borrow `x` as mutable, it is also borrowed as immutable
/// ```
pub fn listing_1_5_noalias(input: &i32, output: &mut i32) {
    if *input == 1 {
        *output = 2;
    }
    if *input != 1 {
        *output = 3;
    }
}

/// Listing 1-6: *Mutability applies only to the immediately referenced memory.*
///
/// With `y: &i32` and `z: &mut &i32`, you may change *which value `y` points
/// at*, because that is the memory `z` immediately refers to. You may not change
/// the value `y` points at, because that lies one indirection further out.
///
/// ```compile_fail,E0594
/// let x = 42;
/// let mut y = &x;
/// let z = &mut y;
/// **z = 84;   // cannot assign to `**z`, which is behind a `&` reference
/// ```
pub fn listing_1_6_mutability_stops_at_one_indirection() {
    let x = 42;
    let other = 84;

    let mut y = &x; // y: &i32
    {
        let z = &mut y; // z: &mut &i32
        *z = &other; // allowed: repoint `y` itself
    }

    assert_eq!(*y, 84);
    assert_eq!(x, 42); // `x` was never reachable for mutation through `z`
}

/// Listing 1-7: *Access through a mutable reference must leave a value behind.*
///
/// The owner of the value is still the caller, and it will drop that value when
/// its own scope ends. So moving the value out through `&mut` would leave the
/// owner believing it still owns something — and at its scope end, it would free
/// the value a second time.
///
/// There are three legal ways to move data around behind the reference:
///
/// * `std::mem::take` — move the value out and leave `Default::default()` behind.
/// * Overwrite — assign a value you already own; the old value is dropped.
/// * `std::mem::swap` — exchange two `&mut` values, leaving both owners holding
///   something legal.
///
/// The version the book comments out is rejected outright, because `*s` would be
/// left empty while the caller still believes it owns a value:
///
/// ```compile_fail,E0507
/// fn replace(s: &mut Box<i32>) {
///     let was = *s;   // cannot move out of `*s` which is behind a mutable reference
/// }
/// ```
///
/// It fails because `s` is a *reference*. Moving out of an owned `Box` would be
/// fine — the box is yours to empty. (That asymmetry is the whole reason the
/// rule exists: only an owner may leave a hole.)
pub fn listing_1_7_must_leave_a_value_behind() {
    fn replace_with_84(s: &mut Box<i32>) {
        // Legal: take the value and leave a default in its place.
        let was = std::mem::take(s);
        assert_eq!(**s, 0); // `take` left `Box<i32>::default()`, so `**s` is legal
        *s = was; // put the old value back

        // Legal: exchange values, so both references end up with something owned.
        let mut r = Box::new(84);
        std::mem::swap(s, &mut r);
        assert_ne!(*r, 84); // `r` now holds what `s` used to
    }

    let mut s = Box::new(42);
    replace_with_84(&mut s);

    // The caller still owns exactly one value and drops it exactly once.
    assert_eq!(*s, 84);
}

/// Overwriting through a `&mut` drops the old value immediately, rather than at
/// the end of the caller's scope. `Arc::strong_count` makes that timing visible.
pub fn overwriting_drops_the_old_value_immediately() {
    let shared = Arc::new(String::from("old"));
    let mut slot = Arc::clone(&shared);

    assert_eq!(*slot, "old"); // the old value is live and readable
    assert_eq!(Arc::strong_count(&shared), 2); // held by `shared` and `slot`

    slot = Arc::new(String::from("new")); // the old value is dropped right here

    // Observable immediately, not at end of scope: overwriting released it.
    assert_eq!(Arc::strong_count(&shared), 1);
    assert_eq!(*slot, "new");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn listing_1_5_noalias_runs() {
        let mut output = 0;
        listing_1_5_noalias(&1, &mut output);
        assert_eq!(output, 2);

        let mut output = 0;
        listing_1_5_noalias(&7, &mut output);
        assert_eq!(output, 3);
    }

    #[test]
    fn listing_1_6_mutability_stops_at_one_indirection_runs() {
        listing_1_6_mutability_stops_at_one_indirection();
    }

    #[test]
    fn listing_1_7_must_leave_a_value_behind_runs() {
        listing_1_7_must_leave_a_value_behind();
    }

    #[test]
    fn overwriting_drops_the_old_value_immediately_runs() {
        overwriting_drops_the_old_value_immediately();
    }
}
