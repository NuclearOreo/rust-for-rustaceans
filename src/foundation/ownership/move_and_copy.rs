//! # Move and copy semantics
//!
//! Assignment looks the same for every type — `let y = x;` — but what it *means*
//! depends on whether the type implements [`Copy`]:
//!
//! * **Not `Copy`** → the value is *moved*. Ownership transfers, and the old
//!   variable becomes unusable, "even though the bits that make up the value are
//!   technically still there."
//! * **`Copy`** → the value is *copied*. "Another identical instance of that same
//!   value is constructed at the destination of the move," and both variables
//!   remain accessible.

use std::mem::size_of;

/// Listing 1-3: *Moving and copying semantics*.
///
/// `x1` is `Copy`, so placing it into the tuple copies it and `x1` survives.
/// `y1` is a `Box`, so placing it into the tuple moves it — which ends
/// `y1`'s usefulness. When the inner scope ends, `z` is dropped, which drops its
/// copy of `42` and the box that came from `y1`; dropping the box also frees the
/// heap memory holding `84`.
///
/// The book's final line, `// let y2 = y1;`, is commented out in the listing
/// because it would not compile. Here it is as a doctest, so the rejection is
/// verified:
///
/// ```compile_fail,E0382
/// let x1 = 42;
/// let y1 = Box::new(84);
/// let _z = (x1, y1);   // y1 is moved into z...
/// let _y2 = y1;        // ...so this is a use of moved value
/// ```
pub fn listing_1_3_move_and_copy() {
    let x1 = 42;
    let y1 = Box::new(84);

    {
        let z = (x1, y1); // x1 is copied in, y1 is moved in

        assert_eq!(z.0, 42);
        assert_eq!(*z.1, 84);
    } // z dropped here: drops its own copy of 42, and the Box (freeing the heap)

    // x1's value is Copy, so it was *not* moved into z. Still fully usable.
    let x2 = x1;
    assert_eq!(x1, 42);
    assert_eq!(x2, 42);
}

/// A move is not "nothing happens": the bits are copied to the new location and
/// the source is then treated as dead. For a `Box` those bits are a heap
/// pointer, so after the move only one of the two variables is allowed to be
/// used — which is what keeps freeing from happening twice.
pub fn a_move_copies_the_bits_but_transfers_ownership() {
    let boxed = Box::new(84u64);
    let heap_address = &*boxed as *const u64;

    let moved = boxed; // ownership transfers; `boxed` is dead from here on

    // Identical bits: the pointer value itself was copied.
    assert_eq!(&*moved as *const u64, heap_address);

    // And a `Box` is nothing but that pointer; ownership, not layout, is what
    // carried the responsibility for the allocation across.
    assert_eq!(size_of::<Box<u64>>(), size_of::<*const u64>());
}

/// `Copy` types are the rebels: assignment copies rather than moves, so the
/// source stays accessible. Most primitives qualify.
pub fn copy_types_stay_usable_after_assignment() {
    let x = 42;
    let y = x; // a copy, not a move

    assert_eq!(x + y, 84); // both are usable
}

/// Passing by value follows the same rule: it moves for non-`Copy` types and
/// copies for `Copy` ones.
///
/// ```compile_fail,E0382
/// fn consume(value: String) -> usize { value.len() }
///
/// let text = String::from("moved in");
/// consume(text);          // `text` moved into the call
/// let again = text.len(); // use of moved value
/// ```
pub fn passing_by_value_moves_or_copies() {
    fn consume_string(value: String) -> usize {
        value.len()
    }

    fn consume_int(value: i32) -> i32 {
        value + 1
    }

    let text = String::from("moved in");
    let len = consume_string(text); // the callee owns `text` now
    assert_eq!(len, 8);

    let number = 42;
    let bumped = consume_int(number); // a copy went in
    assert_eq!(bumped, 43);
    assert_eq!(number, 42); // so `number` is untouched
}

/// Which types are `Copy`? The bar is that "it must be possible to duplicate the
/// type's values simply by copying their bits." Primitive integers and floats
/// clear it; so do tuples, arrays, and `Option`s of `Copy` types; so do shared
/// references, which own nothing.
///
/// ```compile_fail,E0277
/// fn assert_copy<T: Copy>() {}
///
/// assert_copy::<String>();  // the trait `Copy` is not implemented for `String`
/// ```
pub fn which_types_are_copy() {
    fn assert_copy<T: Copy>() {}

    assert_copy::<i32>();
    assert_copy::<f64>();
    assert_copy::<bool>();
    assert_copy::<(u8, u16)>(); // a tuple of Copy types is Copy
    assert_copy::<[u8; 4]>(); // as is an array of a Copy type
    assert_copy::<Option<u8>>(); // `Option<T>` is Copy when `T` is
    assert_copy::<&'static str>(); // a shared reference is Copy: it owns nothing

    // The non-Copy cases, for reference — each of these fails to compile:
    //   assert_copy::<String>();
    //   assert_copy::<Box<u8>>();
    //   assert_copy::<Vec<u8>>();
}

/// Why `Box` cannot be `Copy` — and, more generally, why `Copy` and cleanup are
/// mutually exclusive.
///
/// If `Box` were `Copy`, then `box2 = box1` would leave *both* variables
/// believing they own the same allocation, and both would free it when they went
/// out of scope. "Freeing the memory twice could have catastrophic
/// consequences." The compiler enforces the reasoning at the type level: a type
/// that owns a resource must implement `Drop`, and `Copy` cannot coexist with
/// `Drop`.
///
/// Deriving `Copy` on a type with a non-`Copy` field is rejected first:
///
/// ```compile_fail,E0204
/// #[derive(Copy, Clone)]
/// struct Wrapper(String);   // String is not Copy, so Wrapper cannot be
/// ```
///
/// And adding a destructor to a `Copy` type is rejected outright:
///
/// ```compile_fail,E0184
/// #[derive(Copy, Clone)]
/// struct OwnsAResource;
///
/// impl Drop for OwnsAResource {
///     fn drop(&mut self) {}
/// }
/// ```
///
/// The two errors are two halves of the same rule: copying must be *inert*. A
/// duplicated resource-owning value would duplicate the responsibility to free,
/// and with no `Copy`-and-`Drop` overlap there is no way to write that bug.
pub fn why_box_cannot_be_copy() {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn listing_1_3_move_and_copy_runs() {
        listing_1_3_move_and_copy();
    }

    #[test]
    fn a_move_copies_the_bits_but_transfers_ownership_runs() {
        a_move_copies_the_bits_but_transfers_ownership();
    }

    #[test]
    fn copy_types_stay_usable_after_assignment_runs() {
        copy_types_stay_usable_after_assignment();
    }

    #[test]
    fn passing_by_value_moves_or_copies_runs() {
        passing_by_value_moves_or_copies();
    }

    #[test]
    fn which_types_are_copy_runs() {
        which_types_are_copy();
    }
}
