//! # Propagating Errors — `From` and `Into`
//!
//! > The standard library has many conversion traits, but two of the core ones
//! > are `From` and `Into` . It might strike you as odd to have two: if we have
//! > `From` , why do we need `Into` , and vice versa?
//!
//! The aside gives three reasons, and only two of them are still visible today:
//! the historical coherence one is now a *runnable* two-way conversion
//! ([`Rows`]), the ergonomic one decides which spelling to reach for
//! (`Ident::from` vs `<_ as Into<Ident>>::into`), and the third — `?` uses
//! `From`, not `Into` — is the one that still bites ([`OldError`]).
//!
//! > **FROM AND INTO**
//!
//! > Only since Rust 1.41.0, when the exception for covered types was added to
//! > the coherence rules, are they legal.
//!
//! > if you're writing a method that takes a type that can be turned into a
//! > `Foo` , would you rather write `fn(impl Into<Foo>)` or
//! > `fn<T>(T) where Foo: From<T>` ? And conversely, to turn a string into a
//! > syntax identifier, would you rather write `Ident::from("foo")` or
//! > `<_ as Into<Ident>>::into("foo")` ?
//!
//! > Given that we do have both, you may wonder which you should use in your
//! > code today. The answer, it turns out, is pretty simple: implement `From` ,
//! > and use `Into` in bounds.
//!
//! > The reason is that `Into` has a blanket implementation for any `T` that
//! > implements `From` , so regardless of whether a type explicitly implements
//! > `From` or `Into` , it implements `Into` !
//!
//! The three claims and their evidence:
//!
//! | Claim | Evidence |
//! | --- | --- |
//! | implement `From`, get `Into` free | a `From<u16> for Port` is usable through an `impl Into<Port>` bound, and so is `Port` itself via the std reflexive impl |
//! | two-way conversion was rejected before 1.41 | [`Rows`] has both `From<Vec<T>>` and `From<Rows<T>> for Vec<T>`, and both directions run |
//! | `?` goes through `From` only | [`OldError`] has `Into<NewError>` and no `From`; the `Into` bound accepts it, `?` is `E0277` |

// ---------------------------------------------------------------------------
// Implement `From`, use `Into` in bounds
// ---------------------------------------------------------------------------

/// A syntax identifier, the book's example of a conversion target.
#[derive(Debug, PartialEq, Eq)]
pub struct Ident(String);

impl From<&str> for Ident {
    fn from(text: &str) -> Self {
        Self(text.to_string())
    }
}

/// A port, used to show that the `Into` bound covers `From` impls.
#[derive(Debug, PartialEq, Eq)]
pub struct Port(u16);

impl From<u16> for Port {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

/// > use `Into` in bounds
///
/// > it often significantly easier to use one or the other in different
/// > situations … would you rather write `fn(impl Into<Foo>)` or
/// > `fn<T>(T) where Foo: From<T>` ?
pub fn as_port(value: impl Into<Port>) -> Port {
    value.into()
}

/// The other spelling of the same bound, kept only to show they are equivalent.
pub fn as_port_where<T>(value: T) -> Port
where
    Port: From<T>,
{
    Port::from(value)
}

// ---------------------------------------------------------------------------
// The coherence history
// ---------------------------------------------------------------------------

/// The book's `MyType<T>`: a local wrapper around a `Vec`.
#[derive(Debug, PartialEq, Eq)]
pub struct Rows<T>(Vec<T>);

/// > You can write `impl<T> From<Vec<T>> for MyType<T>` … easily enough
impl<T> From<Vec<T>> for Rows<T> {
    fn from(rows: Vec<T>) -> Self {
        Self(rows)
    }
}

/// > but if you only had `From` _or_ `Into` , you would have to write
/// > `impl<T> From<MyType<T>> for Vec<T>` … However, the compiler used to reject
/// > those implementations!
///
/// Since 1.41 the type parameter `T` is *covered* by the local [`Rows`], so the
/// foreign `Vec<T>` self type is allowed. The uncovered shape,
/// `impl<T> From<T> for Vec<T>`, is still `E0210`; see
/// [`coherence_is_why_this_is_refused`](crate::types::traits_and_trait_bounds::coherence_and_the_orphan_rule::coherence_is_why_this_is_refused).
impl<T> From<Rows<T>> for Vec<T> {
    fn from(rows: Rows<T>) -> Self {
        rows.0
    }
}

// ---------------------------------------------------------------------------
// `?` uses `From`, not `Into`
// ---------------------------------------------------------------------------

/// An error from an old library, which implements `Into<NewError>` directly.
///
/// Before the 1.41 coherence exception, the reverse `impl From<OldError> for
/// NewError` was not legal for a downstream crate, so libraries wrote the
/// `Into` impl instead.
#[derive(Debug)]
pub struct OldError;

/// The error the new code reports.
#[derive(Debug, PartialEq, Eq)]
pub struct NewError;

// `from_over_into` wants this written as `From<OldError> for NewError`, which is
// precisely the impl an old library does not have — and the one whose absence
// makes `?` fail below.
#[allow(clippy::from_over_into)]
impl Into<NewError> for OldError {
    fn into(self) -> NewError {
        NewError
    }
}

/// > it does mean that error types from old libraries that implement `Into`
/// > instead may not work with `?`
///
/// `?` asks for `NewError: From<OldError>`, and there is no such impl:
///
/// ```compile_fail,E0277
/// use rust_for_rustaceans::error_handling::propagating_errors::from_and_into::{
///     NewError, OldError,
/// };
///
/// fn old() -> Result<(), OldError> {
///     Err(OldError)
/// }
///
/// fn load() -> Result<(), NewError> {
///     old()?;   // `NewError: From<OldError>` is not satisfied
///     Ok(())
/// }
/// ```
pub fn an_into_bound_accepts_what_question_mark_rejects() {
    /// > use `Into` in bounds
    ///
    /// This bound is satisfied by the `Into`-only error, so the ergonomic
    /// spelling works where `?` does not.
    fn upgrade(value: impl Into<NewError>) -> NewError {
        value.into()
    }

    fn old() -> Result<(), OldError> {
        Err(OldError)
    }

    assert_eq!(upgrade(OldError), NewError);

    // `?` refuses, but the conversion the old library *did* define is reachable
    // by writing it out.
    let converted = old().map_err(Into::into);
    assert_eq!(converted, Err(NewError));
}

/// > to turn a string into a syntax identifier, would you rather write
/// > `Ident::from("foo")` or `<_ as Into<Ident>>::into("foo")` ?
pub fn from_is_the_readable_call_and_into_is_the_readable_bound() {
    // The two call spellings produce the same value; one is readable.
    let direct = Ident::from("screen");
    let through_into: Ident = <_ as Into<Ident>>::into("screen");
    assert_eq!(direct, through_into);

    // `Into` in a bound, over a `From` impl…
    assert_eq!(as_port(8080u16), Port(8080));

    // …and over the standard library's reflexive `impl<T> From<T> for T`, which
    // is why a `Port` itself is accepted too.
    assert_eq!(as_port(Port(7)), Port(7));

    // The two bound spellings are the same bound.
    assert_eq!(as_port(1u16), as_port_where(1u16));
}

/// > Only since Rust 1.41.0, when the exception for covered types was added to
/// > the coherence rules, are they legal.
///
/// Both directions run: `Vec<T>` → [`Rows`] and [`Rows`]`<T>` → `Vec<T>`.
pub fn a_two_way_conversion_is_legal_today() {
    let rows: Rows<u8> = vec![1, 2, 3].into();
    assert_eq!(rows, Rows(vec![1, 2, 3]));

    let back: Vec<u8> = rows.into();
    assert_eq!(back, [1, 2, 3]);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn an_into_bound_accepts_what_question_mark_rejects_runs() {
        an_into_bound_accepts_what_question_mark_rejects();
    }

    #[test]
    fn from_is_the_readable_call_and_into_is_the_readable_bound_runs() {
        from_is_the_readable_call_and_into_is_the_readable_bound();
    }

    #[test]
    fn a_two_way_conversion_is_legal_today_runs() {
        a_two_way_conversion_is_legal_today();
    }
}
