//! # Representing Errors
//!
//! > When you write code that can fail, the most important question to ask
//! > yourself is how your users will interact with any errors returned. Will
//! > users need to know exactly which error happened and the minutiae about what
//! > went wrong, or will they simply log that an error occurred and move on as
//! > best they can? To understand this, we have to look at whether the nature of
//! > the error is likely to affect what the caller does upon receiving it. This
//! > in turn will dictate how we represent different errors.
//!
//! > You have two main options for representing errors: enumeration and erasure.
//! > That is, you can either have your error type _enumerate_ the possible error
//! > conditions so that the caller can distinguish them, or you can just provide
//! > the caller with a single, _opaque_ error.
//!
//! The section's three subsections, including the special cases it ends on:
//!
//! | Module | Book subsection | What it pins down |
//! | --- | --- | --- |
//! | [`enumeration`] | *Enumeration*, Listing 4-1 | `CopyError::In` vs `CopyError::Out`: a `From` impl cannot label the call site, so the variant is chosen with `map_err`; the `Error`/`Display`/`Debug`/`Send`/`Sync`/`'static` contract around it |
//! | [`opaque_errors`] | *Opaque Errors* | a concrete-but-opaque `DecodeError` (readable line and status, unreachable causes) and a fully erased `BoxError` (composition through `?`, downcasting, `TypeId`, and why errors sit behind a pointer) |
//! | [`special_error_cases`] | *Special Error Cases* | `Result<T, ()>` is not `Option<T>`, `()` is not an `Error`, the never type removes an arm from a match, and `std::thread::Result` is an error made of `dyn Any` |
//!
//! # What is rejection and what is behaviour
//!
//! Most of this section is *behaviour*, because both representations are
//! legitimate and the question is what the caller can do:
//!
//! - enumeration: the same `io::Error` on two streams produces two variants, and
//!   the caller's response differs with the variant;
//! - erasure: three different error types are one `BoxError`, and only
//!   `downcast_ref` gets a concrete one back (and only because it is `'static`);
//! - the special cases: `Option` reports "nothing to return" while `Result<_, ()>`
//!   reports "this failed, consider retrying", `!` makes an arm impossible to
//!   construct, and a panic payload is a `dyn Any` carrying whatever was passed
//!   to `panic!`.
//!
//! The rejections are the four edges of that space, each a `compile_fail`
//! doctest on the item it belongs to: `E0451`/`E0603` for the opaque type's
//! private fields and private cause enum, `E0277` for `Box<dyn Error>` and `()`
//! and a join error where an `Error` is required, `E0521` for a downcast behind
//! a non-`'static` trait object, and `E0308` for `Err(())` in a `Result<_, !>`
//! or an `Option` where a `Result<_, ()>` is expected.

pub mod enumeration;
pub mod opaque_errors;
pub mod special_error_cases;
