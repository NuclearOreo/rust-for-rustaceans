//! # Dynamically Sized Types and Wide Pointers
//!
//! > Most types in Rust implement `Sized` automatically — that is, they have a
//! > size that's known at compile time — but two common types do not: trait
//! > objects and slices.
//!
//! A slice or a trait object has no size the compiler can know, yet you write
//! `&[u8]` and `&dyn Trait` all day. The bridge between the two is the *wide
//! pointer*: a pointer plus one extra word that completes the type.
//!
//! | Book claim | Demo |
//! | --- | --- |
//! | every bound is `Sized` unless you write `?Sized` | [`sized_is_the_default_everywhere`] |
//! | `?Sized` opts out | [`question_mark_sized_accepts_unsized_types`] |
//! | a wide pointer is twice a word | [`a_wide_pointer_is_two_words`] |
//! | a slice's extra word is its length | [`a_slices_extra_word_is_its_length`] |
//! | a trait object's extra word finds the size | [`a_trait_objects_extra_word_is_its_vtable`] |
//! | `Box` and `Arc` store wide pointers too | [`box_and_arc_also_store_wide_pointers`] |

use std::mem::size_of;
use std::mem::size_of_val;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// A pair of types that differ only in size
// ---------------------------------------------------------------------------

/// The trait both concrete types below implement.
pub trait Count {
    fn count(&self) -> usize;
}

/// A 200-byte implementor — deliberately much bigger than a pointer.
pub struct Big(pub [u8; 200]);

impl Count for Big {
    fn count(&self) -> usize {
        200
    }
}

/// An 8-byte implementor: `Small(u8)` is 1 byte, padded… to exactly 1 byte.
pub struct Small(pub u8);

impl Count for Small {
    fn count(&self) -> usize {
        1
    }
}

// ---------------------------------------------------------------------------
// Sized, and the opt-out
// ---------------------------------------------------------------------------

/// > The compiler requires types to be `Sized` nearly everywhere. Struct fields,
/// > function arguments, return values, variable types, and array types must all
/// > be `Sized`.
///
/// Asking for the size of an unsized type is the most direct way to see the
/// requirement: `size_of` takes a `T: Sized`, so `size_of::<[u8]>()` cannot even
/// be written.
///
/// ```compile_fail,E0277
/// let size = std::mem::size_of::<[u8]>();   // the size for values of type `[u8]`
///                                           // cannot be known at compilation time
/// ```
///
/// The same rule fires inside a struct, because a field has no length to store:
///
/// ```compile_fail,E0277
/// struct Corrupt {
///     data: [u8],
///     count: u8,      // only the last field may be unsized
/// }
/// ```
///
/// And on the implicit `Sized` bound of every generic parameter:
///
/// ```compile_fail,E0277
/// fn needs_sized<T>() {}
///
/// needs_sized::<[u8]>();    // `[u8]: Sized` does not hold
/// ```
pub fn sized_is_the_default_everywhere() {
    // What *is* sized: every type with a length baked into it.
    assert_eq!(size_of::<[u8; 3]>(), 3);
    assert_eq!(size_of::<(u8, u8)>(), 2);
    assert_eq!(size_of::<Vec<u8>>(), 3 * size_of::<usize>()); // the buffer is elsewhere
}

/// > every single type bound you write includes `T: Sized` unless you explicitly
/// > opt out of it with `T: ?Sized` (the `?` means "may not be").
///
/// With the bound relaxed, one function body serves a slice, a `str` and a
/// `Vec<u8>`, because all it needs is `as_ref` — the length comes from the wide
/// pointer, not from a compile-time size.
pub fn question_mark_sized_accepts_unsized_types() {
    fn byte_len<T: ?Sized + AsRef<[u8]>>(value: &T) -> usize {
        value.as_ref().len()
    }

    let bytes: &[u8] = &[1, 2, 3];
    assert_eq!(byte_len(bytes), 3); // T = [u8]
    assert_eq!(byte_len("hello"), 5); // T = str
    assert_eq!(byte_len(&vec![0u8; 4]), 4); // T = Vec<u8>
}

// ---------------------------------------------------------------------------
// Wide pointers
// ---------------------------------------------------------------------------

/// > The way to bridge this gap between unsized and sized types is to place
/// > unsized types behind a _wide pointer_ (also known as a _fat pointer_). A
/// > wide pointer is just like a normal pointer, but it includes an extra
/// > word-sized field that gives the additional information about that pointer
/// > that the compiler needs to generate reasonable code for working with the
/// > pointer.
///
/// > crucially, that wide pointer _is_ `Sized`. Specifically, it is twice the
/// > size of a `usize` … one `usize` for holding the pointer, and one `usize`
/// > for holding the extra information needed to "complete" the type.
pub fn a_wide_pointer_is_two_words() {
    let word = size_of::<usize>();

    // Thin pointers: one word.
    assert_eq!(size_of::<&u64>(), word);
    assert_eq!(size_of::<*const u64>(), word);
    assert_eq!(size_of::<Box<u64>>(), word);

    // Wide pointers: two. A slice…
    assert_eq!(size_of::<&[u8]>(), 2 * word);
    assert_eq!(size_of::<*const [u8]>(), 2 * word);
    assert_eq!(size_of::<Box<[u8]>>(), 2 * word);
    assert_eq!(size_of::<&mut [u8]>(), 2 * word);

    // …a string slice, which is just a `[u8]` with UTF-8 in it…
    assert_eq!(size_of::<&str>(), 2 * word);
    assert_eq!(size_of::<String>(), 3 * word); // pointer, length, capacity

    // …and a trait object.
    assert_eq!(size_of::<&dyn Count>(), 2 * word);
    assert_eq!(size_of::<&mut dyn Count>(), 2 * word);
}

/// > For a slice, the extra information is simply the length of the slice.
///
/// The length is not a compile-time fact, so it has to travel with the pointer.
/// Two slices into the same buffer therefore differ *only* in that second word:
/// the data pointer can be identical while the lengths are not.
pub fn a_slices_extra_word_is_its_length() {
    let buffer = [10u8, 20, 30, 40, 50];

    let whole: &[u8] = &buffer;
    let tail: &[u8] = &buffer[2..];
    let shorter: &[u8] = &buffer[2..4];

    // Different lengths, same starting address.
    assert_eq!(whole.as_ptr(), buffer.as_ptr());
    assert_eq!(tail.as_ptr(), shorter.as_ptr());

    // `size_of_val` has no compile-time size to work from; it reads the length
    // out of the wide pointer.
    assert_eq!(size_of_val(whole), 5);
    assert_eq!(size_of_val(tail), 3);
    assert_eq!(size_of_val(shorter), 2);

    // Which is the same number `len()` reports.
    assert_eq!(size_of_val(tail), tail.len());
}

/// > For a trait object — well, we'll get to that later.
///
/// The second word of `&dyn Trait` is a pointer to the *vtable*: the table of
/// function pointers for the concrete type, which includes its size and
/// alignment. That is why two `&dyn Count` of identical width can describe values
/// of wildly different sizes.
pub fn a_trait_objects_extra_word_is_its_vtable() {
    let big: &dyn Count = &Big([0; 200]);
    let small: &dyn Count = &Small(1);

    // Both handles are the same two words, whatever they point at…
    assert_eq!(size_of::<&dyn Count>(), 2 * size_of::<usize>());

    // …and yet `size_of_val` answers correctly for each, because the vtable it
    // reaches through the second word knows the real size.
    assert_eq!(size_of_val(big), 200);
    assert_eq!(size_of_val(small), 1);

    // The vtable is also what makes the method call work.
    assert_eq!(big.count(), 200);
    assert_eq!(small.count(), 1);
}

/// > `Box` and `Arc` also support storing wide pointers, which is why they both
/// > support `T: ?Sized`.
///
/// An owning wide pointer is still two words: the allocation address, and the
/// metadata needed to interpret it.
pub fn box_and_arc_also_store_wide_pointers() {
    let boxed_slice: Box<[u8]> = vec![1, 2, 3].into_boxed_slice();
    let boxed_trait: Box<dyn Count> = Box::new(Big([7; 200]));
    let shared_trait: Arc<dyn Count> = Arc::new(Small(3));

    assert_eq!(size_of::<Box<[u8]>>(), 2 * size_of::<usize>());
    assert_eq!(size_of::<Box<dyn Count>>(), 2 * size_of::<usize>());
    assert_eq!(size_of::<Arc<dyn Count>>(), 2 * size_of::<usize>());

    assert_eq!(boxed_slice.len(), 3);
    assert_eq!(boxed_trait.count(), 200);
    assert_eq!(shared_trait.count(), 1);
}

// ---------------------------------------------------------------------------
// Why the size has to travel with the pointer
// ---------------------------------------------------------------------------

/// An unsized value cannot be moved out of a wide pointer by value: the move
/// would need a destination of a size the compiler does not know.
///
/// ```compile_fail,E0277
/// let boxed: Box<[u8]> = vec![1, 2, 3].into_boxed_slice();
/// let moved: [u8] = *boxed;      // the size for values of type `[u8]` cannot be
///                                // known at compilation time
/// ```
///
/// A user-defined structure can be unsized too, as long as the unsized field is
/// last — the compiler then treats the struct itself as a DST.
///
/// ```compile_fail,E0277
/// struct Header {
///     count: usize,
///     data: [u8],        // legal: trailing unsized field
/// }
///
/// let size = std::mem::size_of::<Header>();   // but `Header` is unsized
/// ```
///
/// Keeping the value *behind* the pointer sidesteps all of it.
pub fn an_unsized_value_cannot_be_moved_out_of_its_pointer() {
    let boxed: Box<[u8]> = vec![1, 2, 3].into_boxed_slice();

    // Behind the pointer, it is usable; `&*boxed` is a `&[u8]`, a wide pointer.
    let borrowed: &[u8] = &boxed;
    assert_eq!(borrowed, &[1, 2, 3]);
    assert_eq!(size_of_val(borrowed), 3);

    // And it can be *moved as a whole* by moving the two-word handle.
    let moved: Box<[u8]> = boxed;
    assert_eq!(moved.len(), 3);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sized_is_the_default_everywhere_runs() {
        sized_is_the_default_everywhere();
    }

    #[test]
    fn question_mark_sized_accepts_unsized_types_runs() {
        question_mark_sized_accepts_unsized_types();
    }

    #[test]
    fn a_wide_pointer_is_two_words_runs() {
        a_wide_pointer_is_two_words();
    }

    #[test]
    fn a_slices_extra_word_is_its_length_runs() {
        a_slices_extra_word_is_its_length();
    }

    #[test]
    fn a_trait_objects_extra_word_is_its_vtable_runs() {
        a_trait_objects_extra_word_is_its_vtable();
    }

    #[test]
    fn box_and_arc_also_store_wide_pointers_runs() {
        box_and_arc_also_store_wide_pointers();
    }

    #[test]
    fn an_unsized_value_cannot_be_moved_out_of_its_pointer_runs() {
        an_unsized_value_cannot_be_moved_out_of_its_pointer();
    }
}
