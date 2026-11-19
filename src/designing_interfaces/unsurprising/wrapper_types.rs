//! # Wrapper Types
//!
//! > Rust does not have object inheritance in the classical sense. However, the
//! > `Deref` trait and its cousin `AsRef` both provide something a little like
//! > inheritance. These traits allow you to have a value of type `T` and call
//! > methods on some type `U` by calling them directly on the `T` -typed value
//! > if `T: Deref<Target = U>` . This feels like magic to the user, and is
//! > generally great.
//!
//! > If you provide a relatively transparent wrapper type (like `Arc` ), there's
//! > a good chance you'll want to implement `Deref` so that users can call
//! > methods on the inner type by just using the `.` operator. If accessing the
//! > inner type does not require any complex or potentially slow logic, you
//! > should also consider implementing `AsRef` … For most wrapper types, you
//! > will also want to implement `From<InnerType>` and `Into<InnerType>` where
//! > possible so that your users can easily add or remove your wrapping.
//!
//! One wrapper, four traits, each with a different promise:
//!
//! | Trait | What it promises | When it is the right choice |
//! | --- | --- | --- |
//! | `Deref` | `.` reaches through to the inner type | the wrapper is transparent |
//! | `AsRef` | `&Wrapper` converts to `&Inner` cheaply | the conversion cannot fail |
//! | `From`/`Into` | the wrapper and the inner type convert by value | cheap to add or remove the wrapping |
//! | `Borrow` | the wrapper *is* a variant of the inner type | map/set keys, and nothing else |
//!
//! The last row is the one the book singles out, because `Borrow` looks like the
//! other three and is not:
//!
//! > `Borrow` is tailored for a much narrower use case: allowing the caller to
//! > supply any one of multiple essentially identical variants of the same type.
//! > It could, perhaps, have been called `Equivalent` instead.
//!
//! > that would not be safe without `Borrow` 's additional requirement that the
//! > target type implements `Hash` , `Eq` , and `Ord` exactly the same as the
//! > implementing type.
//!
//! The last demo breaks that requirement on purpose and watches the lookups fail
//! silently — which is why `Borrow` is not a general-purpose conversion.

use std::borrow::Borrow;
use std::collections::HashSet;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Deref;

// ---------------------------------------------------------------------------
// The transparent wrapper
// ---------------------------------------------------------------------------

/// A transparent wrapper around a `String`: reaching the `str` inside is a field
/// access, never a computation.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Title(String);

impl Title {
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }
}

// ---------------------------------------------------------------------------
// Deref: the dot operator reaches through
// ---------------------------------------------------------------------------

impl Deref for Title {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

/// > This feels like magic to the user, and is generally great.
///
/// Two things become true: every `str` method is callable on `Title`, and a
/// `&Title` coerces to `&str` wherever a `&str` is expected.
pub fn deref_lets_the_inner_types_methods_answer() {
    let title = Title::new("Dune");

    // `len`, `to_uppercase`, and indexing are all `str`'s, reached through `.`.
    assert_eq!(title.len(), 4);
    assert_eq!(title.to_uppercase(), "DUNE");
    assert_eq!(&(*title)[..2], "Du");
    assert_eq!(&*title, "Dune");

    // Deref coercion: `&Title` is accepted where `&str` is declared.
    fn shout(text: &str) -> String {
        text.to_uppercase()
    }
    assert_eq!(shout(&title), "DUNE");
}

// ---------------------------------------------------------------------------
// AsRef: a plain conversion of a reference
// ---------------------------------------------------------------------------

impl AsRef<str> for Title {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// > If accessing the inner type does not require any complex or potentially
/// > slow logic, you should also consider implementing `AsRef` , which allows
/// > users to easily use a `&WrapperType` as an `&InnerType` .
///
/// `AsRef` is what generic code asks for when it only needs to *look* at the
/// inner value. `&Title` gets the impl through std's blanket `impl AsRef<U> for
/// &T where T: AsRef<U>`.
pub fn asref_converts_a_reference_without_the_dot_operator_magic() {
    fn words(text: impl AsRef<str>) -> usize {
        text.as_ref().split_whitespace().count()
    }

    let title = Title::new("Children of Dune");
    assert_eq!(words(&title), 3);
    assert_eq!(words("God Emperor of Dune"), 4);

    // And the conversion itself is just a borrow — no allocation, no failure.
    let inner: &str = title.as_ref();
    assert_eq!(inner, "Children of Dune");
}

// ---------------------------------------------------------------------------
// From / Into: by value, both directions
// ---------------------------------------------------------------------------

impl From<String> for Title {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for Title {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<Title> for String {
    fn from(title: Title) -> Self {
        title.0
    }
}

/// > For most wrapper types, you will also want to implement `From<InnerType>`
/// > and `Into<InnerType>` where possible so that your users can easily add or
/// > remove your wrapping.
///
/// Only `From` is written; `Into` arrives free from std's blanket
/// `impl<T, U> Into<U> for T where U: From<T>`.
pub fn from_and_into_add_and_remove_the_wrapping() {
    let wrapped: Title = "Dune Messiah".into(); // From<&str>
    let from_string: Title = String::from("Heretics of Dune").into(); // From<String>

    let inner: String = wrapped.clone().into(); // From<Title> for String
    assert_eq!(inner, "Dune Messiah");
    assert_eq!(from_string.as_ref(), "Heretics of Dune");

    // `From<&str>` also means `Title::from` is callable as a function.
    let explicit = Title::from("Chapterhouse: Dune");
    assert_eq!(explicit.as_ref(), "Chapterhouse: Dune");
}

// ---------------------------------------------------------------------------
// Borrow: equivalence, not conversion
// ---------------------------------------------------------------------------

/// > `Borrow` also has a blanket implementation of `Borrow<T>` for `T` , `&T` ,
/// > and `&mut T` , which makes it convenient to use in trait bounds to accept
/// > either owned _or_ referenced values of a given type.
///
/// The canonical use: a `HashSet<String>` accepts a `&str` in `contains`,
/// because `std` implements `Borrow<str> for String`.
pub fn borrow_lets_one_type_stand_in_for_an_equivalent_one() {
    let mut tags: HashSet<String> = HashSet::new();
    tags.insert("Dune".to_string());

    // `HashSet<String>::contains<Q>` is declared `where String: Borrow<Q>`,
    // `Q: Hash + Eq` — and `String: Borrow<str>` makes this call legal.
    assert!(tags.contains("Dune"));
    assert!(!tags.contains("Dune Messiah"));

    // The three spellings of one type are interchangeable in a `Borrow` bound.
    fn length<T: Borrow<String>>(value: T) -> usize {
        value.borrow().len()
    }

    let owned = String::from("Dune");
    let shared = &owned;
    let mut mutable = owned.clone();

    assert_eq!(length(owned.clone()), 4); // T = String
    assert_eq!(length(shared), 4); // T = &String
    assert_eq!(length(&mut mutable), 4); // T = &mut String
}

/// The type that treats `String` and `str` as equivalent — but hashes the
/// lowercase form while comparing the original.
#[derive(Debug, PartialEq, Eq)]
pub struct CaseTag(String);

impl CaseTag {
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }
}

impl Hash for CaseTag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_ascii_lowercase().hash(state);
    }
}

impl Borrow<str> for CaseTag {
    fn borrow(&self) -> &str {
        &self.0
    }
}

/// > that would not be safe without `Borrow` 's additional requirement that the
/// > target type implements `Hash` , `Eq` , and `Ord` exactly the same as the
/// > implementing type.
///
/// `Borrow<str>` here is legal — it is not an `unsafe` impl — but it is wrong,
/// because `CaseTag`'s `Hash` and `Eq` disagree with `str`'s. The set does not
/// report an error; it simply stops finding the entry through the borrowed type.
pub fn an_inconsistent_borrow_silently_breaks_lookups() {
    let mut tags: HashSet<CaseTag> = HashSet::new();
    tags.insert(CaseTag::new("Rust"));

    // The type's own notion of identity still works…
    assert!(tags.contains(&CaseTag::new("Rust")));
    assert!(!tags.contains(&CaseTag::new("rust")));

    // …but neither `&str` spelling ever matches: the stored key hashed "rust",
    // and `Borrow<str>` hands back the original "Rust" to compare against.
    assert!(!tags.contains("Rust"));
    assert!(!tags.contains("rust"));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deref_lets_the_inner_types_methods_answer_runs() {
        deref_lets_the_inner_types_methods_answer();
    }

    #[test]
    fn asref_converts_a_reference_without_the_dot_operator_magic_runs() {
        asref_converts_a_reference_without_the_dot_operator_magic();
    }

    #[test]
    fn from_and_into_add_and_remove_the_wrapping_runs() {
        from_and_into_add_and_remove_the_wrapping();
    }

    #[test]
    fn borrow_lets_one_type_stand_in_for_an_equivalent_one_runs() {
        borrow_lets_one_type_stand_in_for_an_equivalent_one();
    }

    #[test]
    fn an_inconsistent_borrow_silently_breaks_lookups_runs() {
        an_inconsistent_borrow_silently_breaks_lookups();
    }
}
