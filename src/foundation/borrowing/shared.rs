//! # Shared References
//!
//! "A shared reference, `&T`, is, as the name implies, a pointer that may be
//! shared. Any number of other references may exist to the same value, and each
//! shared reference is `Copy`, so you can trivially make more of them. Values
//! behind shared references are not mutable."
//!
//! The payoff is a licence granted to the compiler: it "is allowed to assume
//! that the value a shared reference points to *will not change* while that
//! reference lives." Two reads can become one read, because nothing could have
//! written in between — the absence of mutation is guaranteed, not merely
//! unobserved.

use std::ptr;

/// Listing 1-4: *Rust assumes that shared references are immutable.*
///
/// The assertion can never fail. `*input` is read twice, and between those reads
/// nothing is permitted to write to it — `input` is a `&i32`, and `sum` is a
/// distinct `&mut i32`, so the compiler even knows the two cannot overlap.
pub fn listing_1_4_cache(input: &i32, sum: &mut i32) {
    *sum = *input + *input;
    assert_eq!(*sum, 2 * *input);
}

/// Shared references are `Copy`, so "making more of them" costs a pointer copy.
/// All of them necessarily agree, because none can write.
pub fn shared_references_are_copy_and_all_agree() {
    let x = 42;

    let first = &x;
    let second = first; // Copy, not a move: `first` is still usable
    let third = &x;

    assert_eq!(*first, *second);
    assert_eq!(*first, *third);

    // They also point at the same place, not at copies of the value.
    assert!(ptr::eq(first, second));
    assert!(ptr::eq(first, third));
}

/// Because a shared reference is immutable, a function may take several of them
/// and reason about all of them together — none can disturb the others.
///
/// You cannot write through one:
///
/// ```compile_fail,E0594
/// let x = 42;
/// let r = &x;
/// *r = 84;   // cannot assign to `*r`, which is behind a `&` reference
/// ```
///
/// ...and you cannot launder one into a mutable reference. There is no safe cast
/// that upgrades a shared reference to an exclusive one:
///
/// ```compile_fail,E0605
/// let x = 42;
/// let r = &x;
/// let m = r as &mut i32;   // non-primitive cast: `&i32` as `&mut i32`
/// ```
pub fn shared_references_cannot_write() {
    let x = 42;

    fn read_two(a: &i32, b: &i32) -> i32 {
        // Two reads of the same value; the compiler may collapse them.
        *a + *b
    }

    assert_eq!(read_two(&x, &x), 84);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn listing_1_4_cache_runs() {
        let mut sum = 0;
        listing_1_4_cache(&21, &mut sum);
        assert_eq!(sum, 42);
    }

    #[test]
    fn shared_references_are_copy_and_all_agree_runs() {
        shared_references_are_copy_and_all_agree();
    }

    #[test]
    fn shared_references_cannot_write_runs() {
        shared_references_cannot_write();
    }
}
