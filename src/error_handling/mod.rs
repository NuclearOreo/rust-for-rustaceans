//! Chapter 4 — *Error Handling*.
//!
//! > For all but the simplest programs, you will have methods that can fail. In
//! > this chapter, we'll look at different ways to represent, handle, and
//! > propagate those failures and the advantages and drawbacks of each. We'll
//! > start by exploring different ways to represent errors, including
//! > enumeration and erasure, and then examine some special error cases that
//! > require a different representation technique.
//!
//! > It's worth noting that best practices for error handling in Rust are still
//! > an active topic of conversation, and at the time of writing, the ecosystem
//! > has not yet settled on a single, unified approach. This chapter will
//! > therefore focus on the underlying principles and techniques rather than
//! > recommending specific crates or patterns.
//!
//! The chapter has two sections:
//!
//! | Section | Directory | What it pins down |
//! | --- | --- | --- |
//! | *Representing Errors* | [`representing_errors`] | enumeration vs erasure: [`CopyError`](representing_errors::enumeration::CopyError) tells the caller which stream failed, [`DecodeError`](representing_errors::opaque_errors::DecodeError) and [`BoxError`](representing_errors::opaque_errors::BoxError) tell it only that something did — plus the special cases (`Result<T, ()>`, the never type, `std::thread::Result`) |
//! | *Propagating Errors* | [`propagating_errors`] | `?` as unwrap-or-return-early and one `From` hop per conversion; implement `From` / use `Into` in bounds; try blocks (`E0658`) replaced by a closure or a labelled block so cleanup still runs |
//!
//! # How the two representations fail differently
//!
//! Enumeration is a promise about *which* failures the caller can distinguish,
//! so every demo checks the variants and what the caller does with them. Erasure
//! is a promise about *how little* the caller can see, so its demos check the
//! opposite: one type for many causes, downcasting as the only escape hatch, and
//! the two things erasure makes impossible (`Box<dyn Error>` is not an `Error`;
//! a non-`'static` error cannot be downcast). The special cases are each a
//! claim about what the type system already knows: `Option` is not a `Result`,
//! `()` is not an `Error`, a `!` arm cannot be constructed or need not be
//! matched, and a panicking thread returns a `dyn Any` rather than a `dyn Error`.

pub mod propagating_errors;
pub mod representing_errors;
