//! # Types in Memory
//!
//! > Every Rust value has a type. Types serve many purposes in Rust, as we'll
//! > see in this chapter, but one of their most fundamental roles is to tell you
//! > how to interpret bits of memory. For example, the sequence of bits
//! > `0b10111101` (written in hexadecimal notation as `0xBD`) does not mean
//! > anything in and of itself until you assign it a type. When interpreted
//! > under the type `u8`, that sequence of bits is the number 189. When
//! > interpreted under the type `i8`, it is –67.
//!
//! Two questions, asked in order:
//!
//! 1. **Alignment** — which addresses may a value of this type start at?
//! 2. **Layout** — once that is settled, where does each field go inside it?
//!
//! The rest of the section applies the answer to the types you actually use
//! (tuples, arrays, unions, enums) and to the ones that have no single size at
//! all (slices and trait objects).
//!
//! | Module | Book subsection | What it pins down |
//! | --- | --- | --- |
//! | [`alignment`] | *Alignment* | natural alignment, byte boundaries, misaligned access |
//! | [`layout`] | *Layout* | Listing 2-1, and the `repr` attribute |
//! | [`complex_types`] | *Complex Types* | tuple, array, union, enum representation |
//! | [`dynamically_sized_types`] | *Dynamically Sized Types and Wide Pointers* | `Sized`, `?Sized`, wide pointers |
//!
//! # How to read these files
//!
//! Layout facts are asserted with `std::mem::{size_of, align_of, offset_of}`,
//! never eyeballed — `cargo test` re-checks them against the toolchain, which is
//! the only thing that can actually answer the question. Facts that are *not*
//! guaranteed (and therefore cannot be asserted at all) are called out in
//! prose: in particular, `repr(Rust)` is free to change, so the demos record
//! what this compiler does and say so.
//!
//! **Every layout number below is for a 64-bit target**, where a word is 8
//! bytes. `cargo test` on a 32-bit target would fail the `u64` offsets, and it
//! should: those numbers are the point.

pub mod alignment;
pub mod complex_types;
pub mod dynamically_sized_types;
pub mod layout;
