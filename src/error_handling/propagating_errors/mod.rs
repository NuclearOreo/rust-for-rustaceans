//! # Propagating Errors
//!
//! > Rust's `?` operator acts as a shorthand for _unwrap or return early_, for
//! > working easily with errors. But it also has a few other tricks up its
//! > sleeve that are worth knowing about.
//!
//! > Try blocks are also not stable at the time of writing, but there is enough
//! > of a consensus on their usefulness that they're likely to land in a form
//! > similar to that described here.
//!
//! | Module | Book topic | What it pins down |
//! | --- | --- | --- |
//! | [`question_mark`] | the operator | unwrap-or-return-early (the step after the failure never runs), one `From` hop per `?`, and that it follows the *enclosing function's* error type — an `Option` in a `Result` function is `E0277` and vice versa |
//! | [`from_and_into`] | the *From and Into* aside | implement `From`, use `Into` in bounds; the two-way conversion that needed the 1.41 coherence exception; `?` goes through `From`, so an `Into`-only old error is `E0277` even though an `Into` bound accepts it |
//! | [`try_blocks`] | `Try` and try blocks | Listing 4-2 really does skip its cleanup; `try { }` is `E0658`, and a closure or a labelled block gives the same scoping on stable |
//!
//! # What is stable and what is only promised
//!
//! The operator itself is fully stable, so its three mechanics are runnable
//! demos: early return, `From` conversion, and the enclosing-function rule. The
//! two things the book presents as future work are still future work on rustc
//! 1.100, and both are recorded as rejections rather than described:
//!
//! | Feature | Code | Demo |
//! | --- | --- | --- |
//! | `std::ops::Try` (`try_trait_v2`) | `E0658` | `use std::ops::Try;` in [`try_blocks`] |
//! | `try { }` (`try_blocks`) | `E0658` | Listing 4-3 in [`try_blocks`] |
//! | `?` on a custom wrapper | `E0277` | the compiler's help names the nightly-only `Try` |
//!
//! What stable *does* have for the try-block problem is scoping by other means:
//! a closure makes `?` return from the closure, and a labelled block makes the
//! early exit a `break` — which is the book's own description of a try block,
//! "pretty much like a single-iteration loop".

pub mod from_and_into;
pub mod question_mark;
pub mod try_blocks;
