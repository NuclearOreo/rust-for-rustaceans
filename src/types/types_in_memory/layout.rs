//! # Layout
//!
//! > Now that you know about alignment, we can explore how the compiler decides
//! > on the in-memory representation, known as the _layout_, of a type.
//!
//! > By default … the Rust compiler gives very few guarantees about how it lays
//! > out types, which makes for a poor starting point for understanding the
//! > underlying principles. Luckily, Rust provides a `repr` attribute you can
//! > add to your type definitions to request a particular in-memory
//! > representation for that type.
//!
//! This file walks the four representations the section covers, using Listing
//! 2-1's `Foo` as the running example:
//!
//! | `repr` | Promise it buys | Size of `Foo` |
//! | --- | --- | --- |
//! | `repr(C)` | declaration order, C-compatible padding | 32 bytes |
//! | `repr(Rust)` (default) | *none* — the compiler reorders freely | 16 bytes |
//! | `repr(packed)` | no padding, alignment 1 | 16 bytes (5 for the smaller example) |
//! | `repr(align(n))` | at least `n`-byte alignment | n/a — raises alignment |
//! | `repr(transparent)` | same layout as the single inner field | same as the field |
//!
//! The numbers here are asserted, not assumed. They are also *this compiler's*
//! answer: everything except `repr(C)`, `repr(transparent)` and `repr(align)`
//! is deliberately unspecified, so a future toolchain is free to break the
//! `repr(Rust)` demo.
//!
//! A note on spelling: the book writes `#[repr(packed)]`, which means
//! `#[repr(Rust, packed)]`. Clippy wants the ABI named explicitly, so the packed
//! types here are `#[repr(C, packed)]` — the same absence of padding and the
//! same alignment of 1, plus a fixed field order. The `compile_fail` doctests
//! keep the book's spelling, since they are about what the compiler refuses, not
//! about the ABI.

use std::mem::align_of;
use std::mem::offset_of;
use std::mem::size_of;
use std::ptr::addr_of;

// ---------------------------------------------------------------------------
// repr(C): declaration order plus padding
// ---------------------------------------------------------------------------

/// Listing 2-1: alignment affects layout.
///
/// > First the compiler sees the field `tiny`, whose logical size is 1 bit
/// > (`true` or `false`). But since the CPU and memory operate in terms of
/// > bytes, `tiny` is given 1 byte in the in-memory representation.
#[repr(C)]
pub struct Foo {
    pub tiny: bool,
    pub normal: u32,
    pub small: u8,
    pub long: u64,
    pub short: u16,
}

/// `repr(C)` means "lay this out the way a C compiler would": fields in
/// declaration order, each padded up to its own alignment, and the whole thing
/// padded up to the largest alignment it contains.
pub fn listing_2_1_repr_c_pads_to_keep_every_field_aligned() {
    // > the 1 byte we allocated to `tiny` is going to make `normal` miss its
    // > alignment. To rectify this, the compiler inserts 3 bytes of _padding_.
    assert_eq!(offset_of!(Foo, tiny), 0);
    assert_eq!(offset_of!(Foo, normal), 4); // 3 bytes of padding after `tiny`

    // > the current byte offset into the struct is 1 + 3 + 4 = 8. This is already
    // > byte-aligned, so `small` can go immediately after `normal`.
    assert_eq!(offset_of!(Foo, small), 8);

    // > We are now 1 + 3 + 4 + 1 = 9 bytes into `Foo` … so we must insert
    // > another 7 bytes of padding to make `long` aligned again.
    assert_eq!(offset_of!(Foo, long), 16);

    // > This also conveniently ensures the 2-byte alignment we need for the last
    // > field, `short`.
    assert_eq!(offset_of!(Foo, short), 24);

    // > the rule here is to use the largest alignment of any of `Foo`'s fields …
    // > so, to ensure that `Foo` remains aligned if placed in, say, an array, the
    // > compiler then adds a final 6 bytes of padding to make `Foo`'s size a
    // > multiple of its alignment at 32 bytes.
    assert_eq!(align_of::<Foo>(), 8);
    assert_eq!(size_of::<Foo>(), 32);

    // The arithmetic of the padding: 16 bytes of payload, 16 bytes of holes.
    let payload = size_of::<bool>() + size_of::<u32>() + size_of::<u8>();
    let payload = payload + size_of::<u64>() + size_of::<u16>();
    assert_eq!(payload, 16);
    assert_eq!(size_of::<Foo>() - payload, 16); // 3 + 7 + 6
}

// ---------------------------------------------------------------------------
// repr(Rust): the compiler may reorder
// ---------------------------------------------------------------------------

/// The same five fields as [`Foo`], with no `repr` attribute — so the default
/// `repr(Rust)` layout applies.
pub struct FooRust {
    pub tiny: bool,
    pub normal: u32,
    pub small: u8,
    pub long: u64,
    pub short: u16,
}

/// > One of the primary limitations of the C representation is that it requires
/// > that we place all fields in the same order that they appear in the original
/// > struct definition. The default Rust representation `repr(Rust)` removes
/// > that limitation.
///
/// > Since we're now allowed to reorder the fields, we can place them in
/// > decreasing order of size. This means we no longer need the padding between
/// > `Foo`'s fields; the fields themselves are used to achieve the necessary
/// > alignment! `Foo` is now just the size of its fields: only 16 bytes.
///
/// Two things are worth separating here. The *guarantee* is only negative:
/// `repr(Rust)` promises nothing about field order. What the compiler actually
/// does with that freedom — descending size, `long` first, `tiny` last — is an
/// implementation choice, and this demo records it rather than depending on it.
///
/// > even two different types that share all the same fields, of the same type,
/// > in the same order, are not guaranteed to be laid out the same when using
/// > the default Rust layout!
///
/// That claim is untestable by construction: any assertion about the *absence*
/// of a guarantee is a bet on an implementation detail. It is the reason the
/// only safe way to inspect `repr(Rust)` bytes is through `offset_of!` (or FFI
/// `repr(C)`), never through a guessed offset.
pub fn repr_rust_reorders_to_remove_padding() {
    // Declaration order is not preserved: the last-declared-by-size field is
    // first, the first-declared field is last.
    assert!(offset_of!(FooRust, long) < offset_of!(FooRust, tiny));

    // No padding: the struct is exactly as large as its fields.
    let payload = size_of::<bool>() + size_of::<u32>() + size_of::<u8>();
    let payload = payload + size_of::<u64>() + size_of::<u16>();
    assert_eq!(size_of::<FooRust>(), payload);
    assert_eq!(size_of::<FooRust>(), 16);

    // > This is one of the reasons why Rust by default does not give many
    // > guarantees about how a type is laid out in memory: by giving the
    // > compiler more leeway to rearrange things, we can produce more efficient
    // > code.
    assert_eq!(size_of::<FooRust>(), size_of::<Foo>() / 2);
    assert_eq!(align_of::<FooRust>(), align_of::<Foo>()); // same alignment, half the size
}

// ---------------------------------------------------------------------------
// repr(packed): no padding at all
// ---------------------------------------------------------------------------

/// > It turns out there's also a third way to lay out a type, and that is to
/// > tell the compiler that we do not want any padding between our fields.
#[repr(C, packed)]
pub struct PackedFoo {
    pub tiny: bool,
    pub normal: u32,
    pub small: u8,
    pub long: u64,
    pub short: u16,
}

/// The same two fields with and without `repr(packed)`. `Bar` is `repr(C)` so
/// that the padding has a fixed, declarable position: 1 byte of payload, 3 bytes
/// of holes, 4 bytes of payload. `PackedBar` is the 5 bytes of payload alone.
///
/// (Without `repr(C)` the compiler would simply reorder `b` to offset 0, as in
/// [`repr_rust_reorders_to_remove_padding`], and there would be nothing to
/// remove.)
#[repr(C)]
pub struct Bar {
    pub a: u8,
    pub b: u32,
}

#[repr(C, packed)]
pub struct PackedBar {
    pub a: u8,
    pub b: u32,
}

/// > we are willing to take the performance hit of using misaligned accesses
pub fn repr_packed_removes_padding_and_alignment() {
    // Without the attribute: 1 byte, 3 bytes of padding, 4 bytes.
    assert_eq!(size_of::<Bar>(), 8);
    assert_eq!(offset_of!(Bar, b), 4);

    // With it: the fields sit back to back and the type's own alignment drops to
    // 1, because nothing may be assumed about where it starts.
    assert_eq!(align_of::<PackedBar>(), 1);
    assert_eq!(size_of::<PackedBar>(), 5);

    // `Foo` is a special case: its fields add up to exactly 16 bytes, so packing
    // buys alignment (8 → 1) but not size.
    assert_eq!(size_of::<PackedFoo>(), 16);
    assert_eq!(align_of::<PackedFoo>(), 1);
    assert_eq!(size_of::<PackedFoo>(), size_of::<FooRust>());
    assert!(size_of::<PackedFoo>() < size_of::<Foo>());

    // Fields can still be *read* — they are copied out by value, which is legal
    // even when the address they came from was not aligned for their type.
    let packed = PackedBar {
        a: 1,
        b: 0xDEAD_BEEF,
    };
    let (a, b) = (packed.a, packed.b);
    assert_eq!((a, b), (1, 0xDEAD_BEEF));
}

/// > Keep in mind that this may lead to much slower code, and in extreme cases,
/// > this can cause your program to crash if you try to perform operations that
/// > the CPU supports only on aligned arguments.
///
/// The compiler's answer is to make the mistake impossible rather than slow: it
/// refuses to hand out a *reference* to a packed field at all, because a
/// reference would assert an alignment that does not hold.
///
/// ```compile_fail,E0793
/// #[repr(packed)]
/// struct Packed { a: u8, b: u32 }
///
/// fn read(p: &Packed) -> u32 {
///     let reference = &p.b;       // reference to field of packed struct is unaligned
///     *reference
/// }
/// ```
///
/// The same rule catches the most innocent-looking line of all: `println!` takes
/// its arguments by reference, so printing a packed field fails too.
///
/// ```compile_fail,E0793
/// #[repr(packed)]
/// struct Packed { a: u8, b: u32 }
///
/// let p = Packed { a: 1, b: 2 };
/// println!("{}", p.b);            // format arguments are taken by reference
/// ```
pub fn packed_fields_must_be_copied_out_by_value() {
    #[repr(C, packed)]
    struct Packed {
        a: u8,
        b: u32,
    }

    let packed = Packed { a: 1, b: 2 };

    // The legal form: copy each field out, then use the copies.
    let (a, b) = (packed.a, packed.b);
    assert_eq!((a, b), (1, 2));
}

// ---------------------------------------------------------------------------
// repr(align(n)): more alignment than necessary
// ---------------------------------------------------------------------------

/// > Sometimes, you want to give a particular field or type a larger alignment
/// > than it technically requires. You can do that using the attribute
/// > `#[repr(align(n))]`.
#[repr(align(16))]
pub struct Aligned16(pub u8);

/// > A common use case for this is to ensure that different values stored
/// > contiguously in memory (like in an array) end up in different cache lines
/// > on the CPU. That way, you avoid _false sharing_.
#[repr(align(64))]
pub struct CacheLine(pub [u8; 8]);

/// Raising the alignment also raises the size: the padding is not optional,
/// because the *next* element of an array has to be aligned too.
pub fn repr_align_raises_alignment_and_therefore_size() {
    assert_eq!(align_of::<u8>(), 1);
    assert_eq!(size_of::<u8>(), 1);

    assert_eq!(align_of::<Aligned16>(), 16);
    assert_eq!(size_of::<Aligned16>(), 16); // 1 byte of data, 15 bytes of padding

    // A type cannot be smaller than its own alignment.
    assert_eq!(size_of::<Aligned16>() % align_of::<Aligned16>(), 0);
}

/// Two adjacent values, each pinned to a 64-byte block, so a write to one cannot
/// invalidate the other CPU's cache line.
pub fn repr_align_keeps_adjacent_values_on_separate_cache_lines() {
    let pair = [CacheLine([0; 8]), CacheLine([0; 8])];

    assert_eq!(size_of::<CacheLine>(), 64);
    assert_eq!(align_of::<CacheLine>(), 64);

    let first = addr_of!(pair[0]) as usize;
    let second = addr_of!(pair[1]) as usize;

    // Contiguous, one cache line apart — not interleaved.
    assert_eq!(second - first, 64);
    assert_eq!(first % 64, 0);
    assert_ne!(first / 64, second / 64);
}

// ---------------------------------------------------------------------------
// repr(transparent): same layout as the inner field
// ---------------------------------------------------------------------------

/// A newtype with no `repr` attribute. Its layout is `repr(Rust)`, so nothing
/// stops it from differing from a bare `u32` — in this case it does not, but
/// that is luck, not a promise.
pub struct PlainNewtype(pub u32);

/// > Another useful representation is `repr(transparent)`, which can be used
/// > only on types with a single field and which guarantees that the layout of
/// > the outer type is exactly the same as that of the inner type. This comes in
/// > handy in combination with the "newtype" pattern, where you may want to
/// > operate on the in-memory representations of some `struct A` and
/// > `struct NewA(A)` as if they were the same. Without `repr(transparent)`, the
/// > Rust compiler does not guarantee that they will have the same layout.
#[repr(transparent)]
pub struct TransparentNewtype(pub u32);

/// The proof that the outer layout *is* the inner layout: the wrapper's address
/// is the address of its field.
pub fn repr_transparent_is_its_inner_field() {
    assert_eq!(size_of::<TransparentNewtype>(), size_of::<u32>());
    assert_eq!(align_of::<TransparentNewtype>(), align_of::<u32>());

    let wrapper = TransparentNewtype(7);
    let wrapper_address = addr_of!(wrapper) as usize;
    let field_address = addr_of!(wrapper.0) as usize;

    assert_eq!(wrapper_address, field_address);
    assert_eq!(wrapper.0, 7);
}

/// > which can be used only on types with a single field
///
/// ```compile_fail,E0690
/// #[repr(transparent)]
/// struct TwoFields(u32, u8);   // transparent struct needs at most one non-trivial field
/// ```
pub fn repr_transparent_requires_exactly_one_field() {
    // One field: fine. A newtype over every possible payload has one size, the
    // inner type's, with no room for a tag or padding of its own.
    let small = TransparentNewtype(1);
    let large = TransparentNewtype(u32::MAX);

    for value in [&small, &large] {
        assert_eq!(size_of::<TransparentNewtype>(), size_of::<u32>());
        assert_eq!(addr_of!(value.0) as usize, value as *const _ as usize);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn listing_2_1_repr_c_pads_to_keep_every_field_aligned_runs() {
        listing_2_1_repr_c_pads_to_keep_every_field_aligned();
    }

    #[test]
    fn repr_rust_reorders_to_remove_padding_runs() {
        repr_rust_reorders_to_remove_padding();
    }

    #[test]
    fn repr_packed_removes_padding_and_alignment_runs() {
        repr_packed_removes_padding_and_alignment();
    }

    #[test]
    fn packed_fields_must_be_copied_out_by_value_runs() {
        packed_fields_must_be_copied_out_by_value();
    }

    #[test]
    fn repr_align_raises_alignment_and_therefore_size_runs() {
        repr_align_raises_alignment_and_therefore_size();
    }

    #[test]
    fn repr_align_keeps_adjacent_values_on_separate_cache_lines_runs() {
        repr_align_keeps_adjacent_values_on_separate_cache_lines();
    }

    #[test]
    fn repr_transparent_is_its_inner_field_runs() {
        repr_transparent_is_its_inner_field();
    }

    #[test]
    fn repr_transparent_requires_exactly_one_field_runs() {
        repr_transparent_requires_exactly_one_field();
    }
}
