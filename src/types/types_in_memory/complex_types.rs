//! # Complex Types
//!
//! > You might be curious about how the compiler represents other Rust types in
//! > memory. Here's a quick reference:
//! >
//! > **Tuple** Represented like a struct with fields of the same type as the
//! > tuple values in the same order.
//! >
//! > **Array** Represented as a contiguous sequence of the contained type with no
//! > padding between the elements.
//! >
//! > **Union** Layout is chosen independently for each variant. Alignment is the
//! > maximum across all the variants.
//! >
//! > **Enumeration** Same as union, but with one additional hidden shared field
//! > that stores the enum variant discriminant.
//!
//! Each bullet gets one demo. Two of them need a caveat that the book's summary
//! compresses away, and both are noted where they come up:
//!
//! * A tuple is "like a struct" — but a `repr(Rust)` struct, which the compiler
//!   may reorder. So a `(u8, u32, u16)` is 8 bytes, not the 12 a `repr(C)`
//!   struct with the same fields in the same order would take.
//! * An enum's hidden discriminant is not always a whole byte, and not always
//!   present at all: `Option<&T>` is pointer-sized.

use std::mem::align_of;
use std::mem::size_of;
use std::ptr::addr_of;

// ---------------------------------------------------------------------------
// Tuple
// ---------------------------------------------------------------------------

/// The tuple's field types, in declaration order, as a `repr(C)` struct.
#[repr(C)]
pub struct TupleAsCStruct {
    pub first: u8,
    pub second: u32,
    pub third: u16,
}

/// A tuple is a struct whose fields happen to be unnamed. Since it is a
/// `repr(Rust)` struct, that "same order" is a starting point the compiler may
/// depart from — exactly as it does for [`crate::types::types_in_memory::layout::FooRust`].
pub fn a_tuple_is_a_struct_with_unnamed_fields() {
    // Same fields, same alignment as the `repr(C)` version…
    assert_eq!(align_of::<(u8, u32, u16)>(), align_of::<TupleAsCStruct>());
    assert_eq!(align_of::<(u8, u32, u16)>(), 4);

    // …but not the same openness to reordering: the tuple packs to 8 bytes
    // while the `repr(C)` struct is pinned to declaration order and pays 4
    // bytes of padding (3 after `first`, 1 at the end).
    assert_eq!(size_of::<(u8, u32, u16)>(), 8);
    assert_eq!(size_of::<TupleAsCStruct>(), 12);

    // A shorter tuple, where neither layout needs padding.
    assert_eq!(size_of::<(u32, u32)>(), size_of::<[u32; 2]>());
    assert_eq!(size_of::<()>(), 0); // the empty tuple is the unit type
}

// ---------------------------------------------------------------------------
// Array
// ---------------------------------------------------------------------------

/// The field types of [`PackOfFour`], as a `repr(C)` struct.
#[repr(C)]
pub struct PackOfFour {
    pub a: u32,
    pub b: u32,
    pub c: u32,
    pub d: u32,
}

/// > Represented as a contiguous sequence of the contained type with no padding
/// > between the elements.
///
/// "No padding between the elements" is free, because a type's size is already
/// a multiple of its alignment. So `n` elements cost exactly `n × size`, and
/// element `i` starts at `i × size` — never at `i × (size + slack)`.
pub fn an_array_is_contiguous_with_no_padding_between_elements() {
    let numbers = [10u32, 20, 30, 40];

    assert_eq!(size_of::<[u32; 4]>(), 4 * size_of::<u32>());
    assert_eq!(size_of::<[u32; 4]>(), size_of::<PackOfFour>());

    let stride = &numbers[1] as *const u32 as usize - &numbers[0] as *const u32 as usize;
    assert_eq!(stride, size_of::<u32>());

    // The same holds for an element type that *does* contain padding: the
    // padding is inside each element, not added between them.
    let bytes = [0u8; 3];
    let pairs = [(&bytes[0], 1u32), (&bytes[1], 2u32)];
    let pair_stride = addr_of!(pairs[1]) as usize - addr_of!(pairs[0]) as usize;
    assert_eq!(pair_stride, size_of::<(&u8, u32)>());

    // And across the whole array, not just the first two elements.
    assert_eq!(size_of::<[(&u8, u32); 2]>(), 2 * pair_stride);
}

// ---------------------------------------------------------------------------
// Union
// ---------------------------------------------------------------------------

/// > Layout is chosen independently for each variant. Alignment is the maximum
/// > across all the variants.
pub union ThreeBytes {
    pub one: u8,
    pub three: [u8; 3],
}

/// A union whose members want different alignments: 1 byte and 8.
pub union ByteOrWord {
    pub byte: u8,
    pub word: u64,
}

/// A union is the size of its largest member, rounded up to its largest
/// alignment — the members overlap, so only the biggest one has to fit.
///
/// Reading a union field is `unsafe`, because the compiler cannot know which
/// member was last written; that is Chapter 9's subject. Writing one, and asking
/// about its layout, is safe.
pub fn a_union_is_the_size_of_its_largest_member() {
    assert_eq!(size_of::<ThreeBytes>(), 3);
    assert_eq!(align_of::<ThreeBytes>(), 1);

    assert_eq!(size_of::<ByteOrWord>(), 8);
    assert_eq!(align_of::<ByteOrWord>(), align_of::<u64>());

    // Writing any one member is safe. Every member starts at offset 0, which is
    // what "the layout is chosen independently for each variant" amounts to.
    let _word = ByteOrWord { word: 0x0102_0304 };
    let _bytes = ThreeBytes { three: [1, 2, 3] };
}

// ---------------------------------------------------------------------------
// Enumeration
// ---------------------------------------------------------------------------

/// Two variants, no payload: the entire representation is the discriminant.
pub enum Two {
    A,
    B,
}

/// Two variants, one of which carries a payload. `Number`'s payload is 8 bytes,
/// so the tag has to be padded out to stay aligned with it.
pub enum Number {
    Int(i64),
    Float(f64),
}

/// One payload type, three ways of spelling the discriminant. The same variants
/// cost 2, 4 and 8 bytes.
#[repr(u8)]
pub enum Tagged8 {
    A(u8),
    B,
}

#[repr(u16)]
pub enum Tagged16 {
    A(u8),
    B,
}

#[repr(u32)]
pub enum Tagged32 {
    A(u8),
    B,
}

/// > Same as union, but with one additional hidden shared field that stores the
/// > enum variant discriminant. … The size of the discriminant field depends on
/// > the number of variants.
///
/// The tag is a real field with a real cost. Where it lands is up to the
/// compiler, but its *size* follows the number of variants — which is why
/// `#[repr(u8)]`/`#[repr(u16)]`/`#[repr(u32)]` change the size of an enum
/// without touching its data.
pub fn an_enum_is_a_union_plus_a_discriminant() {
    // Nothing but the tag: 1 byte, no alignment requirements.
    assert_eq!(size_of::<Two>(), 1);
    assert_eq!(align_of::<Two>(), 1);

    // The largest payload (8 bytes) plus enough room for the tag, padded to the
    // payload's alignment.
    assert_eq!(size_of::<Number>(), 16);
    assert_eq!(size_of::<Number>(), size_of::<u64>() * 2);

    // Same variants, three discriminant widths.
    assert_eq!(size_of::<Tagged8>(), 2); // 1-byte tag + 1-byte payload
    assert_eq!(size_of::<Tagged16>(), 4); // 2-byte tag + 1 byte + 1 byte padding
    assert_eq!(size_of::<Tagged32>(), 8); // 4-byte tag + 1 byte + 3 bytes padding
}

/// The discriminant is not just storage: it is what `match` reads to decide
/// which variant it is looking at.
pub fn the_discriminant_is_what_matching_reads() {
    fn as_i64(number: Number) -> i64 {
        match number {
            Number::Int(value) => value,
            Number::Float(value) => value as i64,
        }
    }

    let integers = [Number::Int(-3), Number::Int(0), Number::Int(7)];
    let floats = [Number::Float(2.9), Number::Float(-2.9)];

    assert_eq!(integers.map(as_i64), [-3, 0, 7]);
    assert_eq!(floats.map(as_i64), [2, -2]);
}

// ---------------------------------------------------------------------------
// Beyond the section: when the hidden field is not needed
// ---------------------------------------------------------------------------

/// Not a claim from this section, but the natural sequel to "one additional
/// hidden shared field": if a type already has a bit pattern that no valid value
/// can use, that pattern can *be* the discriminant, and the hidden field costs
/// nothing.
///
/// * `&T` can never be null, so `None` is the null pointer.
/// * `Two` only ever holds 0 or 1, so `Some` can be 2, 3, 4, ….
///
/// The compiler performs this "niche optimisation" on its own; there is no
/// attribute for it and no guarantee about which types get it.
pub fn the_discriminant_is_elided_when_a_niche_is_available() {
    assert_eq!(size_of::<Two>(), 1);
    assert_eq!(size_of::<Option<Two>>(), 1); // the niche *is* the discriminant

    let value = 42u8;
    let reference: Option<&u8> = Some(&value);
    assert_eq!(reference.copied(), Some(42));
    assert_eq!(size_of::<Option<&u8>>(), size_of::<&u8>());
    assert_eq!(size_of::<Option<&u8>>(), 8); // just the pointer

    // Where no niche exists, the tag comes back.
    assert_eq!(size_of::<Option<Number>>(), 16);
    assert_eq!(size_of::<Option<bool>>(), 1); // bool has a 254-value niche
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn a_tuple_is_a_struct_with_unnamed_fields_runs() {
        a_tuple_is_a_struct_with_unnamed_fields();
    }

    #[test]
    fn an_array_is_contiguous_with_no_padding_between_elements_runs() {
        an_array_is_contiguous_with_no_padding_between_elements();
    }

    #[test]
    fn a_union_is_the_size_of_its_largest_member_runs() {
        a_union_is_the_size_of_its_largest_member();
    }

    #[test]
    fn an_enum_is_a_union_plus_a_discriminant_runs() {
        an_enum_is_a_union_plus_a_discriminant();
    }

    #[test]
    fn the_discriminant_is_what_matching_reads_runs() {
        the_discriminant_is_what_matching_reads();
    }

    #[test]
    fn the_discriminant_is_elided_when_a_niche_is_available_runs() {
        the_discriminant_is_elided_when_a_niche_is_available();
    }
}
