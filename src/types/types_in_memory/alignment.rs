//! # Alignment
//!
//! > Before we talk about how a type's in-memory representation is determined,
//! > we first need to discuss the notion of _alignment_, which dictates where
//! > the bytes for a type can be stored.
//!
//! Size says how many bytes a value occupies; alignment says which addresses
//! those bytes may start at. Both are queried from `std::mem`, and both are
//! properties of the *type*, not of any particular value.
//!
//! | Book claim | Demo |
//! | --- | --- |
//! | bits mean nothing until a type is assigned | [`the_same_bits_read_as_two_types`] |
//! | values must be at least byte-aligned | [`every_value_starts_at_a_byte_boundary`] |
//! | built-ins are aligned to their size | [`builtins_are_naturally_aligned`] |
//! | complex types take the largest alignment they contain | [`complex_types_take_the_largest_alignment_they_contain`] |
//! | misaligned accesses must be avoided | [`a_misaligned_read_is_done_by_copying_bytes`] |

use std::mem::align_of;
use std::mem::align_of_val;
use std::mem::offset_of;
use std::mem::size_of;

// ---------------------------------------------------------------------------
// Types give bits their meaning
// ---------------------------------------------------------------------------

/// > the sequence of bits `0b10111101` (written in hexadecimal notation as
/// > `0xBD`) does not mean anything in and of itself until you assign it a
/// > type.
///
/// The very same eight bits are 189 or –67 depending on nothing but the type
/// they are read through.
pub fn the_same_bits_read_as_two_types() {
    let as_unsigned: u8 = 0b1011_1101;
    let as_signed: i8 = as_unsigned as i8;

    assert_eq!(as_unsigned, 189);
    assert_eq!(as_unsigned, 0xBD);
    assert_eq!(as_signed, -67);

    // Identical bytes, different meaning: the type is the whole difference.
    assert_eq!(as_unsigned.to_ne_bytes(), as_signed.to_ne_bytes());
}

// ---------------------------------------------------------------------------
// Byte alignment
// ---------------------------------------------------------------------------

/// > all values, no matter their type, must start at a byte boundary. We say
/// > that all values must be at least _byte-aligned_ — they must be placed at an
/// > address that is a multiple of 8 bits.
///
/// In practice *every* value satisfies the stronger condition that its address
/// is a multiple of its own alignment; "byte-aligned" is just the weakest case
/// of that rule, the one a `u8` gets.
pub fn every_value_starts_at_a_byte_boundary() {
    /// The general rule: an address is valid for `T` only if it is a multiple of
    /// `align_of::<T>()`.
    fn is_placed<T>(value: &T) -> bool {
        (value as *const T as usize).is_multiple_of(align_of_val(value))
    }

    let byte = 0xA5u8;
    let word = 0xDEAD_BEEF_DEAD_BEEFu64;
    let pair = (1u16, 2u8);
    let array = [1u32; 4];

    assert!(is_placed(&byte));
    assert!(is_placed(&word));
    assert!(is_placed(&pair));
    assert!(is_placed(&array));

    // Not just the first element: an array's alignment is chosen so that every
    // element lands on a valid address.
    for element in &array {
        assert!(is_placed(element));
    }

    // "pointers point to _bytes_, not _bits_": the smallest step an address can
    // take is one byte, so neighbouring `u8`s differ by exactly 1.
    let bytes = [0u8; 2];
    let step = &bytes[1] as *const u8 as usize - &bytes[0] as *const u8 as usize;
    assert_eq!(step, 1);
}

/// > Built-in values are usually aligned to their size, so a `u8` is
/// > byte-aligned, a `u16` is 2-byte-aligned, a `u32` is 4-byte-aligned, and a
/// > `u64` is 8-byte-aligned.
///
/// That is what "naturally aligned" means: an 8-byte load at an 8-byte-aligned
/// address is a single hardware access, so the rule costs nothing to follow.
pub fn builtins_are_naturally_aligned() {
    assert_eq!(align_of::<u8>(), 1);
    assert_eq!(align_of::<u16>(), 2);
    assert_eq!(align_of::<u32>(), 4);
    assert_eq!(align_of::<u64>(), 8);

    // > A naturally aligned value is one whose alignment matches its size.
    for (size, alignment) in [
        (size_of::<u8>(), align_of::<u8>()),
        (size_of::<u16>(), align_of::<u16>()),
        (size_of::<u32>(), align_of::<u32>()),
        (size_of::<u64>(), align_of::<u64>()),
    ] {
        assert_eq!(size, alignment);
    }

    // > on a 64-bit CPU, most values are accessed in chunks of 8 bytes (64 bits),
    // > with each operation starting at an 8-byte-aligned address. This is
    // > referred to as the CPU's _word size_.
    //
    // The word size is the widest natural alignment, which is why `u64` is the
    // largest built-in that loads in one go on this target.
    assert_eq!(size_of::<usize>(), 8);
    assert_eq!(align_of::<u64>(), size_of::<usize>());
}

/// > Complex types — types that contain other types — are typically assigned
/// > the largest alignment of any type they contain. For example, a type that
/// > contains a `u8`, a `u16`, and a `u32` will be 4-byte-aligned because of the
/// > `u32`.
pub struct Nested {
    pub a: u8,
    pub b: u16,
    pub c: u32,
}

pub fn complex_types_take_the_largest_alignment_they_contain() {
    assert_eq!(align_of::<Nested>(), 4); // because of `c: u32`, not `a: u8`
    assert_eq!(align_of::<Nested>(), align_of::<u32>());

    assert_eq!(align_of::<(u8, u32)>(), 4);
    assert_eq!(align_of::<[u64; 2]>(), 8);
    assert_eq!(align_of::<&u8>(), 8); // a pointer is word-sized, whatever it points at

    // A type's size is always a multiple of its alignment. That is what keeps
    // every element of an array aligned without any per-element bookkeeping.
    for (size, alignment) in [
        (size_of::<Nested>(), align_of::<Nested>()),
        (size_of::<(u8, u32)>(), align_of::<(u8, u32)>()),
        (size_of::<[u64; 2]>(), align_of::<[u64; 2]>()),
        (size_of::<&u8>(), align_of::<&u8>()),
    ] {
        assert!(size.is_multiple_of(alignment));
    }
}

// ---------------------------------------------------------------------------
// Misaligned accesses
// ---------------------------------------------------------------------------

/// A buffer pinned to an 8-byte boundary, so that "one byte into it" is
/// *guaranteed* to be a misaligned address for a `u64`.
#[repr(align(8))]
pub struct AlignedBuffer(pub [u8; 9]);

/// > Operations on data that is not aligned are referred to as _misaligned
/// > accesses_ and can lead to poor performance and bad concurrency problems.
///
/// The safe way to read an unaligned `u64` is to copy its bytes into an aligned
/// `u64`, which compiles down to whatever the hardware needs (on x86 two loads
/// and a splice, on other targets a single unaligned load).
///
/// Reading through a `*const u64` instead is possible, but it is `unsafe` — not
/// because the compiler cannot express it, but because handing the CPU an
/// address it does not want is the programmer's problem. That is Chapter 9's
/// territory; this file stays on the safe side of the line.
pub fn a_misaligned_read_is_done_by_copying_bytes() {
    assert_eq!(offset_of!(AlignedBuffer, 0), 0);

    // Nine bytes: `1u64` in native byte order, but with a leading zero byte so
    // that the interesting eight bytes start one byte *after* the buffer.
    let buffer = AlignedBuffer([0, 1, 0, 0, 0, 0, 0, 0, 0]);

    let misaligned = &buffer.0[1..9];
    assert_eq!(misaligned.len(), 8);

    // The crate's one hard claim about this address: it is not 8-byte-aligned,
    // so a `u64` load from it would be a misaligned access.
    let address = misaligned.as_ptr() as usize;
    assert_eq!(address % align_of::<u64>(), 1);

    // Copying the bytes out is safe, and it is what the compiler is allowed to
    // optimise into a single unaligned load where the target supports one.
    let bytes: [u8; 8] = misaligned.try_into().unwrap();
    assert_eq!(u64::from_ne_bytes(bytes), 1);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn the_same_bits_read_as_two_types_runs() {
        the_same_bits_read_as_two_types();
    }

    #[test]
    fn every_value_starts_at_a_byte_boundary_runs() {
        every_value_starts_at_a_byte_boundary();
    }

    #[test]
    fn builtins_are_naturally_aligned_runs() {
        builtins_are_naturally_aligned();
    }

    #[test]
    fn complex_types_take_the_largest_alignment_they_contain_runs() {
        complex_types_take_the_largest_alignment_they_contain();
    }

    #[test]
    fn a_misaligned_read_is_done_by_copying_bytes_runs() {
        a_misaligned_read_is_done_by_copying_bytes();
    }
}
