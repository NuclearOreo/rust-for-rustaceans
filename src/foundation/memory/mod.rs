//! Chapter 1 — *Talking About Memory*.
//!
//! "Not all memory is created equal." Before the book can talk about the stack,
//! the heap, and static memory, it first establishes a vocabulary. This module
//! follows that structure, one file per subsection:
//!
//! | Module | Book subsection | What it pins down |
//! | --- | --- | --- |
//! | [`terminology`] | *Memory Terminology* | values vs. places vs. variables vs. pointers |
//! | [`variables`] | *Variables in Depth* | the high-level (flows) and low-level (slots) models |
//! | [`regions`] | *Memory Regions* | the stack, the heap, and static memory |
//!
//! # Reading the code
//!
//! Two kinds of example live in these files:
//!
//! * **Runnable demos** — `pub fn`s that assert the book's claims about
//!   observable behaviour. They are all called from their module's `tests`, so
//!   `cargo test` exercises them.
//! * **`compile_fail` doctests** — the book's *illegal* programs (use before
//!   initialization, the conflicting flows of Listing 1-2, dangling references,
//!   borrowing into a spawned thread) cannot live in a compiled crate. They are
//!   written as doctests instead, which `cargo test` also runs: the test passes
//!   only if `rustc` *rejects* the snippet with the expected error code. That
//!   way the borrow checker's complaints are verified, not just described.
//!
//! # The one idea to take away
//!
//! A value's meaning is independent of where its bytes live, and *where* those
//! bytes live — a stack frame, a heap allocation, or static memory — determines
//! how long the value can be reached. Lifetimes, ownership, and borrowing are
//! all downstream of that fact.

pub mod regions;
pub mod terminology;
pub mod variables;
