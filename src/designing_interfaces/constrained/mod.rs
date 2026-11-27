//! # Constrained
//!
//! > Over time, some user will depend on every property of your interface,
//! > whether bug or feature. This is especially true for publicly available
//! > libraries where you have no control over your users. As a result, you should
//! > think carefully before you make user-visible changes.
//!
//! > Many backward incompatible changes are obvious, like renaming a public type
//! > or removing a public method, but some are subtler and tie in deeply with the
//! > way Rust works. Here, we'll cover some of the thornier subtle changes and how
//! > to plan for them. You'll see that you need to balance some of these against
//! > how flexible you want your interface to be— sometimes, something's got to
//! > give.
//!
//! The section's subdivisions, plus the three notes under *Hidden Contracts*,
//! become the files here:
//!
//! | Module | Book subsection | What it pins down |
//! | --- | --- | --- |
//! | [`type_modifications`] | *Type Modifications* | a private field removes a constructor and breaks patterns (`E0423`, `E0027`), tuple → named breaks old patterns (`E0532`), `#[non_exhaustive]` (`E0639`, `E0638`), and `pub(crate)` keeping an item out of the contract (`E0603`) |
//! | [`trait_implementations`] | *Trait Implementations*, Listing 3-5, Listing 3-6 | a new impl making a downstream call ambiguous (`E0034`), a blanket impl conflicting with a downstream one (`E0119`), default vs. required methods (`E0046`), and sealed traits (`E0277`) |
//! | [`reexports`] | *Re-Exports*, Listing 3-7, *The Semver Trick* | two major versions of a dependency are different types (`E0308`), the newtype and `impl Trait` mitigations, and the semver trick that makes one type stand for two |
//! | [`auto_traits`] | *Auto-Traits*, Listing 3-8 | `is_normal::<T>()` as a compile-only test, and `Send`/`Sync`/`Unpin`/`UnwindSafe` promises propagating through `impl Trait` |
//! | [`hidden_items`] | *Hiding Items from Documentation* | `#[doc(hidden)]` is visibility-neutral, so a hidden type's name is out of the contract while its public effects (its auto-traits) are in it |
//!
//! # How "constrained" becomes testable
//!
//! A breaking change is, by definition, a program that used to compile and now
//! does not. That gives the section a single shape for its evidence: a
//! `compile_fail` doctest reproducing the *user's* code — and a doctest is
//! compiled as its own crate, so the user's side of the boundary is real — next
//! to the runnable version that still works. `E0423` for the vanished unit-struct
//! constructor, `E0308` for the two `Empty` types from different major versions,
//! `E0034` for the now-ambiguous method call: each is a user program that the
//! change deletes.
//!
//! The additions point the other way. A new type, a new default method, an impl
//! for a fresh type — those compile for the first time, so they are runnable
//! demos rather than rejections. And where the book's mitigation is what is
//! being demonstrated (`#[non_exhaustive]`, sealed traits, the semver trick), the
//! demo shows the *author's* side compiling the very code the user's side is
//! refused, which is what makes the mitigation a promise rather than a hope.

pub mod auto_traits;
pub mod hidden_items;
pub mod reexports;
pub mod trait_implementations;
pub mod type_modifications;
