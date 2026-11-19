//! # Ergonomic Trait Implementations
//!
//! > Rust does not automatically implement traits for references to types that
//! > implement traits. To phrase this a different way, you cannot generally call
//! > `fn foo<T: Trait>(t: T)` with a `&Bar` , even if `Bar: Trait` . This is
//! > because `Trait` may contain methods that take `&mut self` or `self` , which
//! > obviously cannot be called on `&Bar` . Nonetheless, this behavior might be
//! > very surprising to a user who sees that `Trait` has only `&self` methods!
//!
//! > when you define a new trait, you'll usually want to provide blanket
//! > implementations as appropriate for that trait for `&T where T: Trait` ,
//! > `&mut T where T: Trait` , and `Box<T> where T: Trait` . You may be able to
//! > implement only some of these depending on what receivers the methods of
//! > `Trait` have.
//!
//! Four demos, in the order the surprise escalates:
//!
//! 1. `foo(&bar)` does not compile even though `(&bar).describe()` does — the
//!    asymmetry the section opens with.
//! 2. Blanket impls for `&T`, `&mut T`, and `Box<T>` close it.
//! 3. A trait with a `&mut self` method cannot have the `&T` impl at all — the
//!    receivers decide which of the three are possible.
//! 4. `IntoIterator` for `&T` and `&mut T`, so `for x in &collection` works
//!    without the user writing `.iter()`.

// ---------------------------------------------------------------------------
// The trait, and the surprise
// ---------------------------------------------------------------------------

/// A trait with only a `&self` method — the case the book says is most
/// surprising, because a reference *could* have supported it.
pub trait Describe {
    fn describe(&self) -> String;
}

/// The implementor the user has in hand.
#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    pub title: String,
    pub bpm: u16,
}

impl Describe for Track {
    fn describe(&self) -> String {
        format!("{} at {} BPM", self.title, self.bpm)
    }
}

/// > you cannot generally call `fn foo<T: Trait>(t: T)` with a `&Bar` , even if
/// > `Bar: Trait` .
///
/// The doctest spells the trait out locally so that it really has only the
/// `Bar` implementation — the blanket impls that would rescue `&Bar` are the
/// next demo, and adding them here would hide the surprise.
///
/// ```compile_fail,E0277
/// trait Announce {
///     fn announce(&self) -> String;
/// }
///
/// struct Track;
///
/// impl Announce for Track {
///     fn announce(&self) -> String {
///         String::from("Yours & Mine at 120 BPM")
///     }
/// }
///
/// fn announce_everything<T: Announce>(thing: T) -> String {
///     thing.announce()
/// }
///
/// let track = Track;
/// announce_everything(&track);   // the trait bound `&Track: Announce` is not
///                                // satisfied
/// ```
///
/// The surprise is that the method call one line away *does* work, because `.`
/// inserts the borrows the method needs:
pub fn a_reference_is_not_automatically_an_implementor() {
    // The same local trait, for the half that does compile.
    trait Announce {
        fn announce(&self) -> String;
    }

    impl Announce for Track {
        fn announce(&self) -> String {
            format!("{} at {} BPM", self.title, self.bpm)
        }
    }

    fn announce_everything<T: Announce>(thing: T) -> String {
        thing.announce()
    }

    let track = Track {
        title: "Yours & Mine".to_string(),
        bpm: 120,
    };

    // The bound is written for the owned value…
    let owned = announce_everything(track.clone());
    assert_eq!(owned, "Yours & Mine at 120 BPM");

    // …while `track.announce()` succeeds through auto-deref, even though
    // `&Track` is not an `Announce`. Same method, same value, two different
    // answers to "is this an `Announce`?".
    assert_eq!(track.announce(), owned);
}

// ---------------------------------------------------------------------------
// Blanket impls close the gap
// ---------------------------------------------------------------------------

impl<T: Describe + ?Sized> Describe for &T {
    fn describe(&self) -> String {
        (**self).describe()
    }
}

impl<T: Describe + ?Sized> Describe for &mut T {
    fn describe(&self) -> String {
        (**self).describe()
    }
}

impl<T: Describe + ?Sized> Describe for Box<T> {
    fn describe(&self) -> String {
        (**self).describe()
    }
}

/// > when you define a new trait, you'll usually want to provide blanket
/// > implementations as appropriate for that trait for `&T where T: Trait` ,
/// > `&mut T where T: Trait` , and `Box<T> where T: Trait` .
///
/// Each impl is a one-line delegation, and each unlocks a spelling that
/// previously failed with `E0277`. None of them changes what the method does.
pub fn blanket_impls_for_references_and_boxes_close_the_gap() {
    fn announce<T: Describe>(thing: T) -> String {
        thing.describe()
    }

    let track = Track {
        title: "Helios".to_string(),
        bpm: 96,
    };
    let mut mutable = track.clone();
    let boxed = Box::new(Track {
        title: "Low".to_string(),
        bpm: 72,
    });

    assert_eq!(announce(&track), "Helios at 96 BPM"); // &T
    assert_eq!(announce(&mut mutable), "Helios at 96 BPM"); // &mut T
    assert_eq!(announce(boxed), "Low at 72 BPM"); // Box<T>
}

// ---------------------------------------------------------------------------
// The receivers decide which impls are possible
// ---------------------------------------------------------------------------

/// A trait whose method needs exclusive access.
pub trait Sink {
    fn push(&mut self, value: u8);
}

/// The owned implementor.
#[derive(Debug, Default)]
pub struct Bytes {
    inner: Vec<u8>,
}

impl Bytes {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.inner
    }
}

impl Sink for Bytes {
    fn push(&mut self, value: u8) {
        self.inner.push(value);
    }
}

// A `&mut T` can offer exclusive access, so this impl is possible…
impl<T: Sink + ?Sized> Sink for &mut T {
    fn push(&mut self, value: u8) {
        (**self).push(value);
    }
}

// …and so is the `Box<T>` one.
impl<T: Sink + ?Sized> Sink for Box<T> {
    fn push(&mut self, value: u8) {
        (**self).push(value);
    }
}

/// > You may be able to implement only some of these depending on what receivers
/// > the methods of `Trait` have.
///
/// `&T` is the one that drops out: delegating `push` would have to produce a
/// `&mut T` from behind a shared reference.
///
/// ```compile_fail,E0596
/// trait Sink {
///     fn push(&mut self, value: u8);
/// }
///
/// impl<T: Sink> Sink for &T {
///     fn push(&mut self, value: u8) {
///         T::push(&mut **self, value);   // cannot borrow `**self` as mutable,
///                                        // as it is behind a `&` reference
///     }
/// }
/// ```
pub fn the_receivers_of_a_trait_decide_which_impls_are_possible() {
    fn push_through<S: Sink>(mut sink: S, value: u8) {
        sink.push(value);
    }

    let mut bytes = Bytes::new();
    bytes.push(1);
    push_through(&mut bytes, 2); // S = &mut Bytes

    let mut boxed = Box::new(Bytes::new());
    boxed.push(3); // Sink for Box<T>, through `&mut self`
    push_through(boxed, 4);

    assert_eq!(bytes.into_inner(), vec![1, 2]);
}

// ---------------------------------------------------------------------------
// IntoIterator for references
// ---------------------------------------------------------------------------

/// A collection that implements `IntoIterator` three ways — owned, `&`, `&mut`.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Catalogue {
    entries: Vec<String>,
}

impl Catalogue {
    pub fn new(entries: Vec<String>) -> Self {
        Self { entries }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, String> {
        self.entries.iter()
    }
}

impl IntoIterator for Catalogue {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl<'a> IntoIterator for &'a Catalogue {
    type Item = &'a String;
    type IntoIter = std::slice::Iter<'a, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter()
    }
}

impl<'a> IntoIterator for &'a mut Catalogue {
    type Item = &'a mut String;
    type IntoIter = std::slice::IterMut<'a, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter_mut()
    }
}

/// > For any type that can be iterated over, consider implementing
/// > `IntoIterator` for both `&MyType` and `&mut MyType` where applicable. This
/// > makes `for` loops work with borrowed instances of your type as well out of
/// > the box, just like users would expect.
pub fn into_iterator_for_references_makes_for_loops_just_work() {
    let mut catalogue =
        Catalogue::new(vec!["Kind of Blue".to_string(), "Bitches Brew".to_string()]);

    // `for` over `&Catalogue`: items are `&String`, and the collection survives.
    let borrowed: Vec<&String> = (&catalogue).into_iter().collect();
    assert_eq!(borrowed.len(), 2);

    // `for` over `&mut Catalogue`: the items are `&mut String`, so the loop can
    // edit in place.
    for entry in &mut catalogue {
        entry.push_str(" (1959)");
    }
    assert_eq!(
        catalogue.iter().next().map(String::as_str),
        Some("Kind of Blue (1959)")
    );

    // `for` over the owned collection consumes it last.
    let owned: Vec<String> = catalogue.into_iter().collect();
    assert_eq!(owned[1], "Bitches Brew (1959)");
}

/// The `for` loop a user writes first is over a *reference to* the collection.
/// A type with only the owned impl answers that with `E0277`:
///
/// ```compile_fail,E0277
/// struct Bag(Vec<String>);
///
/// impl IntoIterator for Bag {
///     type Item = String;
///     type IntoIter = std::vec::IntoIter<String>;
///
///     fn into_iter(self) -> Self::IntoIter {
///         self.0.into_iter()
///     }
/// }
///
/// let bag = Bag(vec!["Atlas".to_string()]);
/// for _entry in &bag { }   // `&Bag` is not an iterator
/// ```
pub fn owned_only_into_iterator_rejects_borrowed_loops() {
    let catalogue = Catalogue::new(vec!["Atlas".to_string()]);

    // What the `&Catalogue` impl buys: borrowing the collection in a `for`.
    let mut count = 0;
    for entry in &catalogue {
        count += entry.len();
    }
    assert_eq!(count, 5);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn a_reference_is_not_automatically_an_implementor_runs() {
        a_reference_is_not_automatically_an_implementor();
    }

    #[test]
    fn blanket_impls_for_references_and_boxes_close_the_gap_runs() {
        blanket_impls_for_references_and_boxes_close_the_gap();
    }

    #[test]
    fn the_receivers_of_a_trait_decide_which_impls_are_possible_runs() {
        the_receivers_of_a_trait_decide_which_impls_are_possible();
    }

    #[test]
    fn into_iterator_for_references_makes_for_loops_just_work_runs() {
        into_iterator_for_references_makes_for_loops_just_work();
    }

    #[test]
    fn owned_only_into_iterator_rejects_borrowed_loops_runs() {
        owned_only_into_iterator_rejects_borrowed_loops();
    }
}
