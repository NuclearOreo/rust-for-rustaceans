//! # Flexible — one job, three contracts (Listing 3-1)
//!
//! > Every piece of code you write includes, implicitly or explicitly, a
//! > contract. The contract consists of a set of requirements and a set of
//! > promises. The requirements are restrictions on how the code can be used,
//! > while the promises are guarantees about how the code can be used.
//!
//! > When designing a new interface, you want to think carefully about this
//! > contract. A good rule of thumb is to avoid imposing unnecessary
//! > restrictions and to only make promises you can keep.
//!
//! > In Rust, restrictions usually come in the form of trait bounds and
//! > argument types, and promises come in the form of trait implementations and
//! > return types.
//!
//! Listing 3-1 is the section's opening argument: three signatures that "all
//! take a string and return a string, but they do so under very different
//! contracts."
//!
//! | Signature | Requirement (restriction) | Promise (guarantee) |
//! | --- | --- | --- |
//! | [`frobnicate1`] | an owned `String` — the caller must allocate | an owned `String` — so the callee must allocate too |
//! | [`frobnicate2`] | a `&str` | specifically a `Cow<'_, str>` |
//! | [`frobnicate3`] | anything that is `AsRef<str>` | only that the result is `AsRef<str>` |
//!
//! > None of these function signatures is _better_ than the others. … Instead,
//! > what I want you to take away from this is that you should think carefully
//! > about what contract your interface binds you to, because changing it after
//! > the fact can be disruptive.
//!
//! All three functions below trim the same string, so the only thing that
//! differs between them is the contract. Each demo measures one end of one
//! contract at the call site, and the two `compile_fail` doctests are the two
//! promises being *too narrow* to satisfy the caller.

use std::borrow::Cow;

// ---------------------------------------------------------------------------
// Listing 3-1, signature 1: String in, String out
// ---------------------------------------------------------------------------

/// > The first function requires the caller to own the string in the form of the
/// > `String` type, and it promises that it will return an owned `String` .
///
/// The requirement is visible at the call site: the `String` is moved in, so
/// the caller no longer has it.
///
/// ```compile_fail,E0382
/// use rust_for_rustaceans::designing_interfaces::flexible::contracts::frobnicate1;
///
/// let owned = String::from("  low  ");
/// let trimmed = frobnicate1(owned);
/// assert_eq!(trimmed, "low");
/// assert_eq!(owned, "  low  ");   // use of moved value: `owned`
/// ```
///
/// > Since the contract requires the caller to allocate and requires us to
/// > return an owned `String` , we cannot later make this function
/// > allocation-free in a backward compatible way.
///
/// That is the cost of the promise: the return type is a concrete `String`, so
/// a later version cannot hand back a `&str` instead.
pub fn frobnicate1(s: String) -> String {
    s.trim().to_owned()
}

// ---------------------------------------------------------------------------
// Listing 3-1, signature 2: &str in, Cow out
// ---------------------------------------------------------------------------

/// > The second function relaxes the contract: the caller can provide any
/// > reference to a string, so the user no longer needs to allocate or give up
/// > ownership of a `String` . It also promises to give back a
/// > `std::borrow::Cow` , meaning it can return either a string reference or an
/// > owned `String` , depending on whether it needs to own the string.
///
/// This implementation borrows when there is nothing to trim and owns only the
/// trimmed copy otherwise, so the caller can observe which branch it took. The
/// promise is still narrow in a different way: the return type is nailed down to
/// `Cow`, so an optimized representation arriving later would be a breaking
/// change.
pub fn frobnicate2(s: &str) -> Cow<'_, str> {
    let trimmed = s.trim();
    if trimmed.len() == s.len() {
        Cow::Borrowed(s)
    } else {
        Cow::Owned(trimmed.to_owned())
    }
}

// ---------------------------------------------------------------------------
// Listing 3-1, signature 3: impl AsRef<str> in, impl AsRef<str> out
// ---------------------------------------------------------------------------

/// What [`frobnicate3`] is allowed to hand back: a type that can produce a
/// reference to a string, and nothing more is promised.
pub struct Trimmed(String);

impl AsRef<str> for Trimmed {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// > The third function lifts these restrictions. It requires only that the user
/// > pass in a type that can produce a reference to a string, and it promises
/// > only that the return value can produce a reference to a string.
///
/// The promise is exactly one trait method wide, which is why the caller cannot
/// call anything else on the result — not even `len`:
///
/// ```compile_fail,E0599
/// use rust_for_rustaceans::designing_interfaces::flexible::contracts::frobnicate3;
///
/// let out = frobnicate3("  low  ");
/// let len = out.len();   // no method named `len` found for opaque type `impl AsRef<str>`
/// ```
///
/// The caller gets `.as_ref()` and nothing else. That is the trade: the widest
/// possible acceptance, the narrowest possible promise.
pub fn frobnicate3(s: impl AsRef<str>) -> impl AsRef<str> {
    Trimmed(s.as_ref().trim().to_owned())
}

// ---------------------------------------------------------------------------
// What each contract costs the caller
// ---------------------------------------------------------------------------

/// > Since the contract requires the caller to allocate and requires us to
/// > return an owned `String` , we cannot later make this function
/// > allocation-free in a backward compatible way.
///
/// The first contract is the narrowest and the most expensive on both sides: an
/// owned value goes in and an owned value comes out, so there is an allocation
/// on the way in (the caller's) and one on the way out (ours).
pub fn listing_3_1_owned_in_owned_out() {
    let owned = String::from("  low  ");

    // The type annotation is not decoration: the promise is exactly "a String".
    let trimmed: String = frobnicate1(owned);

    assert_eq!(trimmed, "low");
}

/// > It also promises to give back a `std::borrow::Cow` , meaning it can return
/// > either a string reference or an owned `String` , depending on whether it
/// > needs to own the string.
///
/// The second contract lets the *callee* decide whether to allocate, and the
/// caller can see the decision in the `Cow` it gets back. The `&str` requirement
/// is cheap for a `String` owner too, because `&String` deref-coerces at the
/// call site — what the caller must give up is only the reference's lifetime, not
/// the string.
pub fn listing_3_1_borrowed_in_cow_out() {
    let already_trimmed = String::from("low");
    let borrowed = frobnicate2(&already_trimmed);

    // No allocation: the returned reference points into the caller's buffer.
    assert!(matches!(borrowed, Cow::Borrowed(_)));
    assert!(std::ptr::eq(borrowed.as_ref(), already_trimmed.as_str()));

    let padded = String::from("  low  ");
    let owned = frobnicate2(&padded);

    // Trimming was needed, so the callee had to own a new string.
    assert!(matches!(owned, Cow::Owned(_)));
    assert_eq!(owned, "low");
    assert!(!std::ptr::eq(owned.as_ref(), padded.as_str()));
}

/// A third-party type that happens to be string-like. [`frobnicate3`] accepts
/// it without knowing it exists; [`frobnicate1`] and [`frobnicate2`] cannot.
struct Column<'a>(&'a str);

impl AsRef<str> for Column<'_> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

/// > The third function lifts these restrictions. It requires only that the user
/// > pass in a type that can produce a reference to a string, and it promises
/// > only that the return value can produce a reference to a string.
///
/// Four call sites, one signature: an owned `String` (moved in), a `&String`
/// (still owned by the caller afterwards), a `Cow` that may or may not own, and a
/// custom `AsRef<str>` type the function has never heard of. The output side is
/// just as broad: nothing about `Trimmed` escapes except its `AsRef` impl.
pub fn listing_3_1_as_ref_in_as_ref_out() {
    // By value: the caller chose to give the string up.
    assert_eq!(frobnicate3(String::from("  low  ")).as_ref(), "low");

    // By reference: the caller keeps its string, which is the whole point.
    let kept = String::from("  low  ");
    assert_eq!(frobnicate3(&kept).as_ref(), "low");
    assert_eq!(kept, "  low  ");

    // A Cow, which is neither a String nor a &str.
    let cow: Cow<'_, str> = Cow::Borrowed("  low  ");
    assert_eq!(frobnicate3(cow).as_ref(), "low");

    // A type this module has never seen.
    assert_eq!(frobnicate3(Column("  low  ")).as_ref(), "low");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn listing_3_1_owned_in_owned_out_runs() {
        listing_3_1_owned_in_owned_out();
    }

    #[test]
    fn listing_3_1_borrowed_in_cow_out_runs() {
        listing_3_1_borrowed_in_cow_out();
    }

    #[test]
    fn listing_3_1_as_ref_in_as_ref_out_runs() {
        listing_3_1_as_ref_in_as_ref_out();
    }
}
