//! # Chapter 1 — Foundations
//!
//! "As you dive into the more advanced corners of Rust, it's important that you
//! ensure you have a solid understanding of the fundamentals."
//!
//! Working through the chapter's sections:
//!
//! * [`memory`] — *Talking About Memory*: terminology, the two models of a
//!   variable, and the stack / heap / static regions.
//! * [`ownership`] — *Ownership*: move and copy semantics, dropping, and drop
//!   order.
//! * [`borrowing`] — *Borrowing and Lifetimes*: shared and mutable references,
//!   interior mutability, lifetimes, generic lifetimes, and variance.

pub mod borrowing;
pub mod memory;
pub mod ownership;
