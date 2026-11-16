//! Chapter 2 — *Types*.
//!
//! > Now that the fundamentals are out of the way, we'll look at Rust's type
//! > system. We'll skip past the basics covered in _The Rust Programming
//! > Language_ and instead dive headfirst into how different types are laid out
//! > in memory, the ins and outs of traits and trait bounds, existential types,
//! > and the rules for using types across crate boundaries.
//!
//! The chapter's first section, *Types in Memory*, is the one covered so far.
//! It answers a single question in four pieces: given a type `T`, where do the
//! bytes of a `T` go, and which addresses may they start at?
//!
//! | Module | Book subsection | What it pins down |
//! | --- | --- | --- |
//! | [`types_in_memory::alignment`] | *Alignment* | which addresses a value may start at |
//! | [`types_in_memory::layout`] | *Layout* | `repr(C)`, `repr(Rust)`, `repr(packed)`, `repr(align)`, `repr(transparent)` |
//! | [`types_in_memory::complex_types`] | *Complex Types* | tuples, arrays, unions, enums |
//! | [`types_in_memory::dynamically_sized_types`] | *Dynamically Sized Types and Wide Pointers* | `Sized`, `?Sized`, two-word pointers |
//! | [`traits_and_trait_bounds::compilation_and_dispatch`] | *Compilation and Dispatch* | monomorphization, static vs dynamic dispatch, object safety |
//! | [`traits_and_trait_bounds::generic_traits`] | *Generic Traits* | generic parameters vs associated types |
//! | [`traits_and_trait_bounds::coherence_and_the_orphan_rule`] | *Coherence and the Orphan Rule* | who may implement what |
//! | [`traits_and_trait_bounds::trait_bounds`] | *Trait Bounds* | `where` clauses, derive, associated types, HRTBs |
//! | [`traits_and_trait_bounds::marker_traits`] | *Marker Traits* | `Send`/`Sync`, auto traits, typestates |
//!
//! # The one idea to take away
//!
//! > one of their most fundamental roles is to tell you how to interpret bits of
//! > memory.
//!
//! A type is a contract about interpretation *and* about placement. Rust
//! guarantees very little about placement on purpose — the guarantees it does
//! make are opt-in, and each `repr` attribute trades a promise for something
//! else (C interoperability, density, alignment, or a newtype's identity).

pub mod existential_types;
pub mod traits_and_trait_bounds;
pub mod types_in_memory;
