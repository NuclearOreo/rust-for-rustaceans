//! Chapter 1 — *Borrowing and Lifetimes*.
//!
//! "Rust allows the owner of a value to lend out that value to others, without
//! giving up ownership, through references. *References* are pointers that come
//! with an additional contract for how they can be used."
//!
//! That word — *contract* — is the whole section. A `&T` promises the pointee
//! will not change; a `&mut T` promises exclusivity; a lifetime names the region
//! of code in which the promise has to keep holding. The compiler is allowed to
//! rewrite your program on the strength of those promises, which is why it is
//! strict about enforcing them.
//!
//! | Module | Book subsection |
//! | --- | --- |
//! | [`shared`] | *Shared References* — `&T`, shared, `Copy`, immutable |
//! | [`mutable`] | *Mutable References* — `&mut T`, exclusive, must leave a value |
//! | [`interior_mutability`] | *Interior Mutability* — mutating through `&` |
//! | [`lifetimes`] | *Lifetimes* — the borrow checker, and lifetimes as regions |
//! | [`generic_lifetimes`] | *Generic Lifetimes* — references inside your own types |
//! | [`variance`] | *Lifetime Variance* — covariant, invariant, contravariant |
//!
//! # Code against the contract, not the optimizer
//!
//! "Whether or not the compiler chooses to apply a given optimization is more or
//! less irrelevant. The compiler heuristics change over time, so you generally
//! want to code against what the compiler is *allowed* to do rather than what it
//! actually does in a particular case at a particular moment in time." Every
//! demo here is written against the promise, not against an observed
//! optimisation.
//!
//! # Reading the rejected programs
//!
//! As in [`memory`](super::memory) and [`ownership`](super::ownership), the
//! programs the compiler refuses are `compile_fail` doctests carrying the error
//! code each one produces. Where the section's claim is *"this compiles, and the
//! version with one lifetime fewer does not"*, both halves are present: a
//! runnable demo for the accepted program, a doctest for the rejected one.

pub mod generic_lifetimes;
pub mod interior_mutability;
pub mod lifetimes;
pub mod mutable;
pub mod shared;
pub mod variance;
