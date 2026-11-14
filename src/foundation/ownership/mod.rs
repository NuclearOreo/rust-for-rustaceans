//! Chapter 1 — *Ownership*.
//!
//! "Rust's memory model centers on the idea that all values have a single
//! *owner* — that is, exactly one location (usually a scope) is responsible for
//! ultimately deallocating each value. This is enforced through the borrow
//! checker."
//!
//! Ownership is one rule with three visible consequences, which is why this
//! module has two files rather than one:
//!
//! | Module | Question it answers |
//! | --- | --- |
//! | [`move_and_copy`] | after I hand a value to someone else, may I still use it? |
//! | [`dropping`] | who cleans up, when, in what order, and how often? |
//!
//! # The rules
//!
//! 1. Every value has exactly one owner.
//! 2. Assigning, passing by value, pushing to a vector, or boxing a value
//!    *moves* ownership to the new location. The old variable stays dead even
//!    though its bits may still be sitting in memory.
//! 3. Types that implement [`Copy`] opt out: they are *copied* instead, and both
//!    locations stay usable. To qualify, a type must be duplicable by copying
//!    its bits — so it may not contain a non-`Copy` type and may not own a
//!    resource requiring cleanup.
//! 4. The owner drops its value when it goes out of scope. Because there is one
//!    owner, cleanup cannot happen twice; a reference does not own its referent,
//!    so dropping a reference drops nothing.
//!
//! # Two kinds of example
//!
//! As in [`memory`](super::memory): runnable demos assert the observable
//! behaviour, while the programs the compiler must *reject* (use after move,
//! deriving `Copy` on a type that owns a resource) are `compile_fail` doctests.

pub mod dropping;
pub mod move_and_copy;
