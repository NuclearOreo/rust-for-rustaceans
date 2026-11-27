//! # Obvious — Documentation
//!
//! > The first step to making your interfaces transparent is to write good
//! > documentation. I could write an entire book dedicated to how to write
//! > documentation, but let's focus on Rust-specific advice here.
//!
//! > First, clearly document any cases where your code may do something
//! > unexpected, or where it relies on the user doing something beyond what's
//! > dictated by the type signature. Panics are a good example of both of these
//! > circumstances: if your code can panic, document that fact, along with the
//! > circumstances it might panic under. Similarly, if your code might return an
//! > error, document the cases in which it does. For unsafe functions, document
//! > what the caller must guarantee in order for the call to be safe.
//!
//! > Second, include end-to-end usage examples for your code on a crate and
//! > module level. These are more important than examples for specific types or
//! > methods, since they give the user a feel for how everything fits together.
//!
//! > **NOTE** _Very method-specific examples that show that, yes, the `len`
//! > method indeed returns the length are unlikely to tell the user anything new
//! > about your code._
//!
//! The end-to-end example therefore lives here, at the top of the module, and
//! not on any one method. Because it is a doctest it is also a test:
//!
//! ```
//! use rust_for_rustaceans::designing_interfaces::obvious::documentation::api::{
//!     first_line, Limit, Reader, ReaderError,
//! };
//!
//! // A limit of two lines, and input that fits.
//! let limit = Limit::new(2).expect("the limit is non-zero");
//! let reader = Reader::new("alpha\nbeta", limit);
//! assert_eq!(reader.read().unwrap(), ["alpha", "beta"]);
//! assert_eq!(first_line("alpha\nbeta"), "alpha");
//!
//! // The two documented failures, in the order the docs list them.
//! let empty = Reader::new("", limit);
//! assert_eq!(empty.read(), Err(ReaderError::Empty));
//!
//! let crowded = Reader::new("alpha\nbeta\ngamma", limit);
//! assert_eq!(crowded.read(), Err(ReaderError::TooMany { limit: 2 }));
//! ```
//!
//! > Third, organize your documentation. Having all your types, traits, and
//! > functions in a single top-level module makes it difficult for the user to
//! > get a sense of where to start. Take advantage of modules to group together
//! > semantically related items. Then, use intra-documentation links to
//! > interlink items. If the documentation on type A talks about trait B, then
//! > it should link to that trait right there.
//!
//! That is why the readable surface sits in [`api`] instead of being spread over
//! this module: the links above (`[`api::Reader`]`, `[`api::Limit`]`) are how the
//! reader gets from here to there, and `cargo doc --no-deps` is what proves they
//! still resolve.
//!
//! > Also consider marking parts of your interface that are not intended to be
//! > public but are needed for legacy reasons with `#[doc(hidden)]` , so that
//! > they do not clutter up your documentation.
//!
//! > And finally, enrich your documentation wherever possible. … Use
//! > `#[doc(cfg(..))]` to highlight items that are available only under certain
//! > configurations so the user quickly realizes why some method that's listed
//! > in the documentation isn't available.
//!
//! > Use `#[doc(alias = "...")]` to make types and methods discoverable under
//! > other names that users may search for them by.
//!
//! Each of those three attributes needs its own note below, because only two of
//! them are stable today.

// ---------------------------------------------------------------------------
// The documented surface: one module, one error, one documented panic
// ---------------------------------------------------------------------------

/// The API the module example above walks through.
///
/// The two `clippy` lints enabled here are the reason this module is more than
/// prose: with `missing_errors_doc` and `missing_panics_doc` on, a
/// `Result`-returning method without an `# Errors` section — or a function that
/// can panic without a `# Panics` section — breaks the repo's "zero clippy
/// warnings" rule. The documentation is part of the verification loop, not just
/// a comment.
pub mod api {
    #![warn(clippy::missing_errors_doc, clippy::missing_panics_doc)]

    use std::num::NonZeroUsize;

    /// How many lines [`Reader`] will hand back.
    ///
    /// A zero limit is rejected by [`Limit::new`], so every `Limit` that exists is
    /// usable — a restriction encoded in the constructor instead of a panic at
    /// the point of use.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Limit(NonZeroUsize);

    impl Limit {
        /// A limit of `lines`, or `None` if `lines` is zero.
        pub fn new(lines: usize) -> Option<Self> {
            NonZeroUsize::new(lines).map(Self)
        }

        /// The number of lines allowed.
        pub fn get(self) -> usize {
            self.0.get()
        }
    }

    /// Why [`Reader::read`] returned an error.
    ///
    /// An associated value per variant is what makes the `# Errors` section
    /// checkable: [`ReaderError::TooMany`] carries the limit it violated, so the
    /// caller can report the right number without guessing.
    #[derive(Debug, PartialEq, Eq)]
    pub enum ReaderError {
        /// The input was empty, so there was nothing to read.
        Empty,
        /// The input had more lines than the [`Limit`] allowed.
        TooMany {
            /// The limit that was exceeded.
            limit: usize,
        },
    }

    /// Splits a string into lines, refusing more than its [`Limit`].
    #[derive(Debug)]
    pub struct Reader<'a> {
        input: &'a str,
        limit: Limit,
    }

    impl<'a> Reader<'a> {
        /// A reader over `input` that will return at most `limit` lines.
        pub fn new(input: &'a str, limit: Limit) -> Self {
            Self { input, limit }
        }

        /// The lines of the input, up to the limit.
        ///
        /// # Errors
        ///
        /// Returns [`ReaderError::Empty`] if the input is empty — the type has no
        /// way to say "zero lines" without also saying "that was a mistake".
        ///
        /// Returns [`ReaderError::TooMany`] if the input has more lines than the
        /// [`Limit`], carrying the limit in the error so the caller does not have
        /// to remember what it asked for.
        pub fn read(&self) -> Result<Vec<&'a str>, ReaderError> {
            if self.input.is_empty() {
                return Err(ReaderError::Empty);
            }

            let lines: Vec<&'a str> = self.input.lines().collect();
            if lines.len() > self.limit.get() {
                return Err(ReaderError::TooMany {
                    limit: self.limit.get(),
                });
            }
            Ok(lines)
        }
    }

    /// The first line of `input`.
    ///
    /// # Panics
    ///
    /// Panics if `input` is empty: an empty input has no first line, and this
    /// function has no way to say so in its return type. Use [`Reader::read`]
    /// when emptiness is a possibility worth handling.
    pub fn first_line(input: &str) -> &str {
        input.lines().next().expect("empty input has no first line")
    }
}

// ---------------------------------------------------------------------------
// A documented error is the error that happens
// ---------------------------------------------------------------------------

/// > Similarly, if your code might return an error, document the cases in which
/// > it does.
///
/// An `# Errors` section is a claim about observable behaviour, so it is
/// checkable: each documented case is produced here, and the associated value is
/// the documented one. A happy path is included so the variants are not
/// mistaken for the only outcomes.
pub fn documented_errors_are_the_errors_that_occur() {
    let limit = api::Limit::new(2).unwrap();

    let empty = api::Reader::new("", limit);
    assert_eq!(empty.read(), Err(api::ReaderError::Empty));

    let crowded = api::Reader::new("alpha\nbeta\ngamma", limit);
    assert_eq!(crowded.read(), Err(api::ReaderError::TooMany { limit: 2 }));

    let fits = api::Reader::new("alpha\nbeta", limit);
    assert_eq!(fits.read().unwrap(), ["alpha", "beta"]);
}

/// Renders a panic payload as text, whichever of the two shapes it arrived in.
fn panic_message(payload: &(dyn std::any::Any + Send)) -> &str {
    payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&'static str>().copied())
        .unwrap_or("a panic payload that is not a string")
}

/// > Panics are a good example of both of these circumstances: if your code can
/// > panic, document that fact, along with the circumstances it might panic
/// > under.
///
/// The `# Panics` section says *if the input is empty*, and that is exactly the
/// condition that panics — with the message the section promises. The condition
/// is asserted here by catching the panic, which also keeps the demo to the
/// "returns normally, asserts internally" shape every other file uses.
pub fn a_documented_panic_is_the_panic_that_occurs() {
    let payload = std::panic::catch_unwind(|| api::first_line("")).unwrap_err();
    let message = panic_message(payload.as_ref());

    assert!(message.contains("empty input has no first line"));

    // The documented non-panicking case, for the same reason as above.
    assert_eq!(api::first_line("alpha\nbeta"), "alpha");
}

// ---------------------------------------------------------------------------
// # [doc(hidden)]: public for compatibility, hidden from the docs
// ---------------------------------------------------------------------------

/// > Also consider marking parts of your interface that are not intended to be
/// > public but are needed for legacy reasons with `#[doc(hidden)]` , so that
/// > they do not clutter up your documentation.
///
/// This is a *documentation* attribute, not a visibility change: the item stays
/// `pub` and stays callable, which is the point — legacy callers keep working
/// while new users are not offered the item. `cargo doc` omits it from the
/// rendered pages, so the clutter it removes is genuinely from the docs and not
/// from the API.
#[doc(hidden)]
pub fn legacy_line_count(input: &str) -> usize {
    input.lines().count()
}

/// > Use `#[doc(alias = "...")]` to make types and methods discoverable under
/// > other names that users may search for them by.
///
/// Search aliases do not appear in the rendered signature and have no runtime
/// effect, so there is nothing to assert about them beyond "the item still
/// works" — the payoff is that a user who types `lines` or `line count` into
/// rustdoc's search box finds [`count_lines`].
///
/// The third enrichment the book suggests, `#[doc(cfg(..))]`, is still
/// nightly-only, so it can only be shown as a rejection:
///
/// ```compile_fail,E0658
/// // the `doc(cfg)` and `doc(auto_cfg)` attributes are experimental
/// #[doc(cfg(feature = "csv"))]
/// pub fn from_csv() {}
/// ```
#[doc(alias = "lines")]
#[doc(alias = "line count")]
pub fn count_lines(input: &str) -> usize {
    legacy_line_count(input)
}

/// The two attributes above, used from the same place a user would.
///
/// `#[doc(hidden)]` does not make an item private and `#[doc(alias)]` does not
/// make it behave differently, so both are callable here exactly as before. What
/// the pair changes is what a *reader* sees, which is why the demonstration is
/// "still works" plus the note that rustdoc filters and indexes them.
pub fn hidden_and_aliased_items_are_still_items() {
    assert_eq!(legacy_line_count("alpha\nbeta"), 2);
    assert_eq!(count_lines("alpha\nbeta\ngamma"), 3);
    assert_eq!(count_lines(""), 0);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn documented_errors_are_the_errors_that_occur_runs() {
        documented_errors_are_the_errors_that_occur();
    }

    #[test]
    fn a_documented_panic_is_the_panic_that_occurs_runs() {
        a_documented_panic_is_the_panic_that_occurs();
    }

    #[test]
    fn hidden_and_aliased_items_are_still_items_runs() {
        hidden_and_aliased_items_are_still_items();
    }
}
