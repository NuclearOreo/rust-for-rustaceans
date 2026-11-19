//! # Naming Practices
//!
//! > A user of your interface will encounter it first through its names; they
//! > will immediately start to infer things from the names of types, methods,
//! > variables, fields, and libraries they come across.
//!
//! > A method called `iter` probably takes `&self` , and probably gives you an
//! > iterator. A method called `into_inner` probably takes `self` and likely
//! > returns some kind of wrapped type. A type called `SomethingError` probably
//! > implements `std::error::Error` and appears in various `Result` s.
//!
//! > A corollary to this is that things that share a name _should_ in fact work
//! > the same way.
//!
//! Three conventions, each checked by what the name makes possible rather than
//! by prose:
//!
//! | Name | What the user guesses | How it is verified |
//! | --- | --- | --- |
//! | `iter` | takes `&self`, yields references | the collection is usable after iterating |
//! | `into_inner` | takes `self`, returns the inner type | calling it twice is `E0382` |
//! | `…Error` | implements `std::error::Error`, appears in `Result` | `?` converts it into `Box<dyn Error>` |
//!
//! The corollary is demonstrated with two unrelated types that expose the same
//! two names: code written against the convention works on both without either
//! type knowing about the other.

use std::error::Error;
use std::fmt;

// ---------------------------------------------------------------------------
// Two collections that follow the naming convention
// ---------------------------------------------------------------------------

/// A collection of band names. It exists to own the conventional accessors.
#[derive(Debug, Default)]
pub struct Bands {
    names: Vec<String>,
}

impl Bands {
    /// `new` is the second-most expected name in Rust; it returns `Self`.
    pub fn new(names: Vec<String>) -> Self {
        Self { names }
    }

    /// > A method called `iter` probably takes `&self` , and probably gives you
    /// > an iterator.
    ///
    /// The returned iterator borrows from `self`, which is why `self` is not
    /// consumed and the item type is a reference.
    pub fn iter(&self) -> std::slice::Iter<'_, String> {
        self.names.iter()
    }

    /// > A method called `into_inner` probably takes `self` and likely returns
    /// > some kind of wrapped type.
    pub fn into_inner(self) -> Vec<String> {
        self.names
    }
}

/// A second, deliberately unrelated type with the same two names. Nothing links
/// `Setlist` to `Bands`; only the convention does.
#[derive(Debug, Default)]
pub struct Setlist {
    names: Vec<String>,
}

impl Setlist {
    pub fn new(names: Vec<String>) -> Self {
        Self { names }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, String> {
        self.names.iter()
    }

    pub fn into_inner(self) -> Vec<String> {
        self.names
    }
}

// ---------------------------------------------------------------------------
// `iter` borrows and yields references
// ---------------------------------------------------------------------------

/// The signature of `iter`, not its documentation, is the convention: `&self`
/// in, references out.
pub fn iter_borrows_and_yields_references() {
    let bands = Bands::new(vec!["Low".to_string(), "Helios".to_string()]);

    // A reference comes out, so the item type is `&String`…
    let first: Option<&String> = bands.iter().next();
    assert_eq!(first.map(String::as_str), Some("Low"));

    // …and `&self` goes in, so the collection survives the iteration.
    let all: Vec<&String> = bands.iter().collect();
    assert_eq!(all.len(), 2);
    assert_eq!(bands.into_inner().len(), 2);
}

// ---------------------------------------------------------------------------
// `into_*` consumes
// ---------------------------------------------------------------------------

/// > A method called `into_inner` probably takes `self` and likely returns some
/// > kind of wrapped type.
///
/// `into_` is the conventional prefix for "consumes the receiver and gives the
/// parts back", which makes a second call a use-after-move:
///
/// ```compile_fail,E0382
/// struct Bands(Vec<String>);
///
/// impl Bands {
///     fn into_inner(self) -> Vec<String> {
///         self.0
///     }
/// }
///
/// let bands = Bands(vec![String::from("Low")]);
/// let _names = bands.into_inner();
/// let _again = bands.into_inner();   // use of moved value: `bands`
/// ```
pub fn into_inner_consumes_and_unwraps() {
    let bands = Bands::new(vec!["Low".to_string(), "Helios".to_string()]);

    let names = bands.into_inner();

    // The wrapper is gone; only the inner type remains.
    assert_eq!(names, vec!["Low".to_string(), "Helios".to_string()]);
    assert_eq!(names.len(), 2);
}

// ---------------------------------------------------------------------------
// `…Error` is an error
// ---------------------------------------------------------------------------

/// A type whose name promises `std::error::Error`.
#[derive(Debug, PartialEq, Eq)]
pub struct TempoError {
    /// The offending input, quoted back to the user.
    pub input: String,
}

impl fmt::Display for TempoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}` is not a tempo in beats per minute", self.input)
    }
}

impl Error for TempoError {}

/// The parser that returns the error in a `Result`, as the name suggests.
pub fn parse_tempo(input: &str) -> Result<u16, TempoError> {
    input.trim().parse().map_err(|_| TempoError {
        input: input.to_string(),
    })
}

/// > A type called `SomethingError` probably implements `std::error::Error` and
/// > appears in various `Result` s.
///
/// The payoff is not `Display` or `Debug` on their own — it is that `?` can
/// convert the error into anything that implements `From<TempoError>`, including
/// the boxed error trait object every `main` and test already returns.
///
/// The negative half is a type that only *looks* like an error. It has nothing
/// for `?` to convert, so the boxed error type named in the signature is not
/// reachable:
///
/// ```compile_fail,E0277
/// struct NotAnError;
///
/// fn load() -> Result<(), Box<dyn std::error::Error>> {
///     Err(NotAnError)?;   // `?` couldn't convert the error:
///                         // `NotAnError: Error` is not satisfied
///     Ok(())
/// }
/// ```
pub fn an_error_type_is_an_error_in_the_question_mark_sense() {
    assert_eq!(parse_tempo("120"), Ok(120));
    assert_eq!(parse_tempo(" 90 "), Ok(90));

    let error = parse_tempo("fast").unwrap_err();
    assert_eq!(
        error.to_string(),
        "`fast` is not a tempo in beats per minute"
    );
    assert_eq!(error.input, "fast");

    /// The `?` conversion needs `TempoError: Error + 'static`, nothing else.
    fn load(input: &str) -> Result<u16, Box<dyn Error>> {
        let tempo = parse_tempo(input)?;
        Ok(tempo)
    }

    let boxed = load("fast").unwrap_err();
    assert!(boxed.downcast_ref::<TempoError>().is_some());
}

// ---------------------------------------------------------------------------
// The corollary: shared names, shared shape
// ---------------------------------------------------------------------------

/// > A corollary to this is that things that share a name _should_ in fact work
/// > the same way.
///
/// The type annotations are the check: if either type's `iter` yielded owned
/// `String`s, or its `into_inner` returned something else, these bindings would
/// be `E0308`. Because the names agree, code written against one shape works
/// against both — and the user who guessed correctly was rewarded.
pub fn shared_names_work_the_same_way() {
    let bands = Bands::new(vec!["Low".to_string(), "Helios".to_string()]);
    let setlist = Setlist::new(vec!["Low".to_string(), "Helios".to_string()]);

    let from_bands: Vec<&String> = bands.iter().collect();
    let from_setlist: Vec<&String> = setlist.iter().collect();
    assert_eq!(from_bands, from_setlist);

    let band_names: Vec<String> = bands.into_inner();
    let song_names: Vec<String> = setlist.into_inner();
    assert_eq!(band_names, song_names);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn iter_borrows_and_yields_references_runs() {
        iter_borrows_and_yields_references();
    }

    #[test]
    fn into_inner_consumes_and_unwraps_runs() {
        into_inner_consumes_and_unwraps();
    }

    #[test]
    fn an_error_type_is_an_error_in_the_question_mark_sense_runs() {
        an_error_type_is_an_error_in_the_question_mark_sense();
    }

    #[test]
    fn shared_names_work_the_same_way_runs() {
        shared_names_work_the_same_way();
    }
}
