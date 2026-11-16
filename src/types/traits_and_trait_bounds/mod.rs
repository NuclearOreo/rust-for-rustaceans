//! # Traits and Trait Bounds
//!
//! > Traits are a key piece of Rust's type system — they are the glue that
//! > allows types to interoperate even though they don't know about each other
//! > at the time they are defined. …
//! >
//! > we're going to take a look at some of the more technical aspects of
//! > traits: how they're implemented, restrictions you have to adhere to, and
//! > some more esoteric uses of traits.
//!
//! | Module | Book subsection | What it pins down |
//! | --- | --- | --- |
//! | [`compilation_and_dispatch`] | *Compilation and Dispatch* | monomorphization, static vs dynamic dispatch, vtables, object safety |
//! | [`generic_traits`] | *Generic Traits* | `trait Foo<T>` vs `trait Foo { type Bar; }` |
//! | [`coherence_and_the_orphan_rule`] | *Coherence and the Orphan Rule* | who may implement what, and the exemptions |
//! | [`trait_bounds`] | *Trait Bounds* | `where` clauses, derive, associated types, HRTBs (Listings 2-2 … 2-6) |
//! | [`marker_traits`] | *Marker Traits* | `Send`/`Sync`, auto traits, marker types |
//!
//! # How to demonstrate "you may not"
//!
//! Half of this section is about *rules*, which is to say about code the
//! compiler rejects. Two techniques carry those demos:
//!
//! * **`compile_fail` doctests** for the rejections, each naming its error code:
//!   `E0038` for a trait that cannot be a trait object, `E0117` and `E0210` for
//!   the orphan rule, `E0119` for a conflicting implementation, `E0223` for an
//!   ambiguous associated type, `E0597` for a missing higher-ranked bound.
//! * **A doctest is its own crate.** That is what makes the orphan-rule listings
//!   testable at all: inside `rust_for_rustaceans`, everything the crate defines
//!   is *local*, so nothing can be a violation. Inside a doctest, this crate's
//!   items are as foreign as `std`'s, which is precisely the situation the rule
//!   is about — so a doctest can `use rust_for_rustaceans::…` and then
//!   legitimately be rejected for implementing a foreign trait on a foreign
//!   type. [`coherence_and_the_orphan_rule::ForeignTrait`] exists only for that.
//!
//! Not every rejection has an error code: calling a `where Self: Sized` method
//! through a trait object is refused with a plain `error:` and no `E…`, so that
//! one is demonstrated by its positive half (the trait *is* object-safe) and
//! described in prose.

pub mod coherence_and_the_orphan_rule;
pub mod compilation_and_dispatch;
pub mod generic_traits;
pub mod marker_traits;
pub mod trait_bounds;
