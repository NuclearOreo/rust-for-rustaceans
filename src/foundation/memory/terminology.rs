//! # Memory Terminology
//!
//! Four words that the rest of the book leans on:
//!
//! * A **value** is "the combination of a type and an element of that type's
//!   domain of values." It is what you *meant*; its bytes are merely its
//!   *representation*.
//! * A **place** is "a location that can hold a value" — a stack slot, a heap
//!   allocation, static memory, and so on.
//! * A **variable** is the most common place: "a named value slot on the stack."
//! * A **pointer** is "a value that holds the address of a region of memory," so
//!   it *points to* a place and can be dereferenced to reach the value there.
//!
//! The subtle part is that a variable holds a *value*, and a pointer is itself a
//! value — so assigning one pointer variable to another copies the address, not
//! the pointee.

use std::mem::size_of;
use std::mem::size_of_val;
use std::ptr;

/// A value's meaning is independent of its representation: the same value always
/// has the same bytes, wherever those bytes happen to be stored.
///
/// The `u8` value `6` is the mathematical integer six; its representation is the
/// single byte `0x06`. The `str` value `"Hello world"` is a string; its
/// representation is its UTF-8 encoding.
pub fn representation_differs_from_meaning() {
    let six: u8 = 6;
    assert_eq!(six.to_ne_bytes(), [0x06]);

    let hello: &str = "Hello world";
    // ...and the UTF-8 encoding of the string value is just those bytes.
    assert_eq!(hello.as_bytes(), b"Hello world");
    assert_eq!(hello.as_bytes()[0], 0x48); // 'H'
}

/// Listing 1-1: *Values, variables, and pointers*.
///
/// There are four values here — `42`, `43`, the address of `x`, and the address
/// of `y` — and four variables. `var1` and `var2` "store separate, independent
/// copies of that value," which is why reassigning `var2` leaves `var1` alone.
///
/// The rule doing the work is that `=` stores the value of the right-hand side
/// into the *place* named by the left-hand side. It never writes through a
/// pointer, even when the value being stored happens to be one.
pub fn listing_1_1_values_variables_pointers() {
    let x = 42;
    let y = 43;

    let var1 = &x;
    let mut var2 = &x; // a second, independent copy of "the address of x"

    assert!(ptr::eq(var1, var2));

    var2 = &y; // overwrite var2's own slot; x is untouched

    assert_eq!(*var1, 42);
    assert_eq!(*var2, 43);
    assert!(!ptr::eq(var1, var2));
}

/// A pointer points to a *place*, and any number of variables may store that
/// same pointer value, all reaching the one value stored there.
pub fn many_pointers_one_place() {
    let x = 42;
    let a = &x;
    let b = &x;
    let c = a; // copies the pointer value, not the `i32`

    assert!(ptr::eq(a, b));
    assert!(ptr::eq(a, c));
    assert_eq!(*a + *b + *c, 126);
}

/// `let string = "Hello world";` does **not** put `"Hello world"` in `string`.
///
/// The variable's actual value is a pointer to the first character of the string
/// value — plus the length, as the book's note foreshadows (that pair is the
/// "wide pointer" of Chapter 2). So the stack slot is two words wide, while the
/// eleven bytes of text live elsewhere: in static memory, since a string literal
/// is a `&'static str`.
pub fn a_string_variable_holds_a_pointer() {
    let string = "Hello world";

    // Half one of the wide pointer: the address of the first character.
    let first_char = string.as_ptr();
    assert_eq!(first_char, string.as_bytes().as_ptr());
    assert!(!first_char.is_null());

    // Half two: the length.
    assert_eq!(string.len(), 11);

    // Together they are two `usize`s — far more than the eleven bytes of text.
    // The variable's slot is the size of a `&str` (a wide pointer)...
    assert_eq!(size_of::<&str>(), 2 * size_of::<usize>());

    // ...while the thing it points at is just the eleven bytes of text.
    assert_eq!(size_of_val(string), 11);

    // The bytes outlive the frame because they are in static memory, which is
    // exactly what `'static` promises.
    let static_ref: &'static str = string;
    assert_eq!(static_ref.as_bytes()[6], b'w');
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn representation_differs_from_meaning_runs() {
        representation_differs_from_meaning();
    }

    #[test]
    fn listing_1_1_runs() {
        listing_1_1_values_variables_pointers();
    }

    #[test]
    fn many_pointers_one_place_runs() {
        many_pointers_one_place();
    }

    #[test]
    fn a_string_variable_holds_a_pointer_runs() {
        a_string_variable_holds_a_pointer();
    }
}
