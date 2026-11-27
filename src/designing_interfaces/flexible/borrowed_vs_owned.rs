//! # Borrowed vs. Owned
//!
//! > For nearly every function, trait, and type you define in Rust, you must
//! > decide whether it should own, or just hold a reference to, its data.
//! > Whatever decision you make will have far-reaching implications for the
//! > ergonomics and performance of your interface.
//!
//! > If the code you write needs ownership of the data, such as to call methods
//! > that take `self` or to move the data to another thread, it must store the
//! > owned data. When your code must own data, it should generally also make the
//! > caller provide owned data, rather than taking values by reference and
//! > cloning them. This leaves the caller in control of allocation, and it is
//! > upfront about the cost of using the interface in question.
//!
//! > On the other hand, if your code doesn't need to own the data, it should
//! > operate on references instead.
//!
//! The demos walk the decision in the order the book does:
//!
//! | Case | Interface | What the caller observes |
//! | --- | --- | --- |
//! | ownership is conditional | [`lossy`] → `Cow<'_, str>` | borrowed for valid UTF-8, owned only when a replacement is needed |
//! | ownership is needed | [`WordIndex::insert`] takes a `String` | the caller's allocation is moved in, not duplicated |
//! | ownership is not needed | [`WordIndex::count`] takes a `&str` | no allocation, and the caller's string survives |
//! | the type is small | by value | a reference would be *larger* than the value |
//!
//! The last one is the exception the book warns about, and the file ends with
//! the cost on the other side: a borrowed field infects every user of the type
//! with a lifetime, which is one reason to own data even when a borrow would do.

use std::borrow::Cow;
use std::collections::HashMap;
use std::mem::size_of;

// ---------------------------------------------------------------------------
// Ownership conditional on the data: Cow
// ---------------------------------------------------------------------------

/// > For example, `String::from_utf8_lossy` needs to take ownership of the byte
/// > sequence that is passed to it only if it contains invalid UTF-8 sequences.
/// > In this case, the `Cow` type is your friend: it lets you operate on
/// > references if the data allows, and it lets you produce an owned value if
/// > necessary.
///
/// The callee cannot know in advance which branch it will take; `Cow` is what
/// lets the return type say "one or the other" without a second allocation on
/// the happy path.
pub fn lossy(bytes: &[u8]) -> Cow<'_, str> {
    String::from_utf8_lossy(bytes)
}

/// > it lets you operate on references if the data allows, and it lets you
/// > produce an owned value if necessary.
///
/// The caller can see which branch was taken, and only the invalid input costs an
/// allocation — the same function, one contract, two costs.
pub fn a_cow_borrows_when_it_can_and_owns_only_when_it_must() {
    let valid = b"ferris";
    let borrowed = lossy(valid);

    assert!(matches!(borrowed, Cow::Borrowed(_)));
    assert_eq!(borrowed, "ferris");
    // No copy: the `&str` still points into the caller's byte slice.
    assert!(std::ptr::eq(borrowed.as_ref().as_ptr(), valid.as_ptr()));

    let invalid = b"ferris \xff";
    let owned = lossy(invalid);

    assert!(matches!(owned, Cow::Owned(_)));
    assert_eq!(owned, "ferris \u{fffd}");
    // A replacement character had to be written, so a new buffer exists.
    assert!(!std::ptr::eq(owned.as_ref().as_ptr(), invalid.as_ptr()));
}

// ---------------------------------------------------------------------------
// Ownership needed: the caller allocates
// ---------------------------------------------------------------------------

/// A word counter. It stores its words, so it needs owned `String`s.
#[derive(Debug, Default)]
pub struct WordIndex {
    entries: HashMap<String, usize>,
}

impl WordIndex {
    /// > When your code must own data, it should generally also make the caller
    /// > provide owned data, rather than taking values by reference and cloning
    /// > them. This leaves the caller in control of allocation, and it is upfront
    /// > about the cost of using the interface in question.
    ///
    /// Taking the `String` by value means the caller can move an existing
    /// allocation in — and if it would rather not give its string up, it calls
    /// `.clone()` itself, where the cost is visible.
    pub fn insert(&mut self, word: String) {
        *self.entries.entry(word).or_insert(0) += 1;
    }

    /// The version the book warns about. It takes a borrow and clones inside, so
    /// the allocation happens anyway, just out of sight.
    pub fn insert_cloned(&mut self, word: &str) {
        self.insert(word.to_owned());
    }

    /// > On the other hand, if your code doesn't need to own the data, it should
    /// > operate on references instead.
    ///
    /// Looking a word up only reads the map, so it takes a `&str`: `&String` and
    /// `&str` both work at the call site, and nothing is allocated.
    pub fn count(&self, word: &str) -> usize {
        self.entries.get(word).copied().unwrap_or(0)
    }

    /// Where the *stored* copy of `word` lives, which is what makes the
    /// difference between moving and cloning observable.
    pub fn stored_buffer(&self, word: &str) -> Option<*const u8> {
        self.entries
            .get_key_value(word)
            .map(|(key, _)| key.as_ptr())
    }
}

/// > This leaves the caller in control of allocation, and it is upfront about
/// > the cost of using the interface in question.
///
/// Both halves are measured against the caller's own buffer: passing the `String`
/// by value moves that buffer into the index (same address), while passing a
/// `&str` and letting the callee clone creates a second one (different address).
/// The lookup side allocates nothing at all.
pub fn the_caller_decides_whether_the_allocation_happens() {
    let mut index = WordIndex::default();

    // Moved in: the buffer the caller built is the buffer the index now owns.
    let owned = String::from("ferris");
    let caller_buffer = owned.as_ptr();
    index.insert(owned);
    assert_eq!(index.stored_buffer("ferris"), Some(caller_buffer));

    // Borrowed in, cloned inside: a second allocation the caller cannot see.
    let borrowed = String::from("corro");
    let caller_buffer = borrowed.as_ptr();
    index.insert_cloned(&borrowed);
    assert_ne!(index.stored_buffer("corro"), Some(caller_buffer));

    // Lookups allocate nothing, and the caller keeps both strings usable.
    assert_eq!(index.count("ferris"), 1);
    assert_eq!(index.count(&borrowed), 1);
    assert_eq!(borrowed, "corro");
}

// ---------------------------------------------------------------------------
// The exception: small types, and Copy types that are not small
// ---------------------------------------------------------------------------

/// > One common exception to this rule is with small types like `i32` , `bool` ,
/// > or `f64` , which are just as cheap to store and copy directly as to store
/// > through references.
///
/// The rule of thumb is about the size of the *value*, not about `Copy`:
///
/// > Be wary of assuming this holds true for all `Copy` types, though; `[u8;
/// > 8192]` is `Copy` , but it would be expensive to store and copy it all over
/// > the place.
pub fn checksum_by_value(bytes: [u8; 8192]) -> u8 {
    bytes[0]
}

pub fn checksum_by_reference(bytes: &[u8; 8192]) -> u8 {
    bytes[0]
}

/// The small-type exception and the `Copy`-is-not-cheap warning, side by side.
///
/// A `u16` is 2 bytes and a `&u16` is 8, so taking the value by value is
/// cheaper — the exception. An `[u8; 8192]` is 8 KiB and a reference to it is 8
/// bytes, even though the array is `Copy` and therefore accepted by value
/// without a `.clone()` to warn the reader.
pub fn copy_does_not_mean_cheap() {
    assert_eq!(size_of::<u16>(), 2);
    assert_eq!(size_of::<&u16>(), size_of::<usize>());
    assert!(size_of::<&u16>() > size_of::<u16>());

    assert_eq!(size_of::<[u8; 8192]>(), 8192);
    assert_eq!(size_of::<&[u8; 8192]>(), size_of::<usize>());

    // `Copy` means this compiles although the bytes go by value; the copy is
    // silent, which is exactly the trap.
    let data = [7u8; 8192];
    assert_eq!(checksum_by_value(data), checksum_by_reference(&data));
    assert_eq!(data, [7u8; 8192]); // still usable — a copy, not a move
}

// ---------------------------------------------------------------------------
// The other cost: lifetimes in the interface
// ---------------------------------------------------------------------------

/// A configuration borrowed from the caller. Free to build, and it hands a
/// lifetime to every value that holds it.
pub struct BorrowedConfig<'a> {
    pub name: &'a str,
}

/// The owning twin, which allocates once and carries no lifetime.
pub struct OwnedConfig {
    pub name: String,
}

pub fn borrowed_config(name: &str) -> BorrowedConfig<'_> {
    BorrowedConfig { name }
}

pub fn owned_config(name: impl Into<String>) -> OwnedConfig {
    OwnedConfig { name: name.into() }
}

/// > Other times, reference lifetimes complicate the interface so much that it
/// > becomes a pain to use. If your users are struggling to get code to compile
/// > on top of your interface, that's a sign that you may want to (even
/// > unnecessarily) take ownership of certain pieces of data.
///
/// A borrowed field cannot outlive its source, so the borrowing version cannot
/// be stored past the block that owns the string:
///
/// ```compile_fail,E0597
/// use rust_for_rustaceans::designing_interfaces::flexible::borrowed_vs_owned::borrowed_config;
///
/// let config;
/// {
///     let name = String::from("demo");
///     config = borrowed_config(&name);
/// }   // `name` does not live long enough
///
/// assert_eq!(config.name, "demo");
/// ```
///
/// The owning version costs one allocation and no lifetime:
pub fn owning_removes_the_lifetime_from_the_callers_problem() {
    let config;
    {
        let name = String::from("demo");
        config = owned_config(name); // the string moves in
    }
    assert_eq!(config.name, "demo");

    // The borrowing version is fine as long as the source outlives the config —
    // which is the whole reason the lifetime has to be written down.
    let name = String::from("demo");
    let borrowed = borrowed_config(&name);
    assert_eq!(borrowed.name, "demo");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn a_cow_borrows_when_it_can_and_owns_only_when_it_must_runs() {
        a_cow_borrows_when_it_can_and_owns_only_when_it_must();
    }

    #[test]
    fn the_caller_decides_whether_the_allocation_happens_runs() {
        the_caller_decides_whether_the_allocation_happens();
    }

    #[test]
    fn copy_does_not_mean_cheap_runs() {
        copy_does_not_mean_cheap();
    }

    #[test]
    fn owning_removes_the_lifetime_from_the_callers_problem_runs() {
        owning_removes_the_lifetime_from_the_callers_problem();
    }
}
