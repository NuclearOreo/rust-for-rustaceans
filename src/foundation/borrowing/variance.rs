//! # Lifetime Variance
//!
//! "Variance is a concept that programmers are often exposed to but rarely know
//! the name of because it's mostly invisible." It describes which types may be
//! used in place of which others: "a type `A` is a subtype of another type `B` if
//! `A` is at least as useful as `B`."
//!
//! For lifetimes the rule reads: if `'b: 'a` (`'b` outlives `'a`), then `'b` is a
//! subtype of `'a`. A `'static` reference is more useful than a short one,
//! because it can be used anywhere a short one can.
//!
//! | Variance | Meaning | Example |
//! | --- | --- | --- |
//! | Covariant | subtype may stand in for supertype | `&'a T` in `'a`, and in `T` |
//! | Invariant | the type must match exactly | `&mut T` in `T`, `Cell<T>` in `T` |
//! | Contravariant | *supertype* may stand in for subtype | `Fn(T)` in `T` |
//!
//! The practical consequence: mutability forces invariance, and invariance
//! reduces flexibility. Listing 1-11 is the case where that bites, and where the
//! fix is to split one lifetime into two.

// ---------------------------------------------------------------------------
// Covariance: `&'a T` is covariant in both `'a` and `T`
// ---------------------------------------------------------------------------

/// A `&'static str` may be used wherever a shorter `&'a str` is expected: `'static`
/// outlives `'a`, so it is the subtype, and `&'a T` is covariant in `'a`.
pub fn covariance_in_the_lifetime() {
    fn needs_any_str(reference: &str) -> usize {
        reference.len()
    }

    let static_ref: &'static str = "static";
    assert_eq!(needs_any_str(static_ref), 6);

    let owned = String::from("shorter");
    assert_eq!(needs_any_str(&owned), 7); // both work: covariance goes one way
}

/// `&'a T` is covariant in `T` too, so a `&Vec<&'static str>` can be passed to a
/// function expecting a `&Vec<&'a str>`.
// `&Vec` rather than `&[_]` because the book's example is specifically
// `&Vec<&'static str>`; the covariance being shown is identical either way.
#[allow(clippy::ptr_arg)]
pub fn covariance_in_the_type() {
    fn needs_any_vec(references: &Vec<&str>) -> usize {
        references.len()
    }

    let statics: Vec<&'static str> = vec!["a", "b"];
    assert_eq!(needs_any_vec(&statics), 2);
}

// ---------------------------------------------------------------------------
// Invariance: `&mut T` (and `Cell<T>`) is invariant in `T`
// ---------------------------------------------------------------------------

/// You cannot pass a `&mut Vec<&'static str>` where a `&mut Vec<&'a str>` is
/// wanted. If you could, the callee could push a short-lived reference into the
/// vector, and the caller — still believing it owns a `Vec<&'static str>` — would
/// carry on using a dangling one.
///
/// ```compile_fail,E0597
/// fn push_short<'a>(v: &mut Vec<&'a str>, s: &'a str) {
///     v.push(s);
/// }
///
/// let mut v: Vec<&'static str> = vec!["static"];
/// let short = String::from("short");
/// push_short(&mut v, &short);   // `short` does not live long enough
/// ```
///
/// Any type that offers mutation is invariant for the same reason, which is why
/// `Cell<T>` is invariant in `T` as well.
pub fn invariance_blocks_writing_a_shorter_lifetime_in() {
    // The same shape, but with matching lifetimes throughout: fine.
    fn push_short<'a>(v: &mut Vec<&'a str>, s: &'a str) {
        v.push(s);
    }

    let text = String::from("short");
    let mut v: Vec<&str> = Vec::new();
    push_short(&mut v, &text);

    assert_eq!(v.len(), 1);
}

// ---------------------------------------------------------------------------
// Contravariance: `Fn(T)` is contravariant in `T`
// ---------------------------------------------------------------------------

/// A function is "more useful if it's okay with its arguments being *less*
/// useful." Something that accepts *any* `&str` can be used where something that
/// demands a `&'static str` is required — the relationship is flipped relative to
/// the argument type on its own, which is what makes it contravariance.
pub fn contravariance_in_function_arguments() {
    fn accepts_anything(reference: &str) -> usize {
        reference.len()
    }

    // `fn(&str)` coerces to `fn(&'static str)`: the looser function satisfies
    // the stricter requirement.
    let stricter: fn(&'static str) -> usize = accepts_anything;

    assert_eq!(stricter("static"), 6);
}

/// The reverse does not hold: a function that insists on `'static` cannot stand
/// in for one that must accept arbitrary short-lived references.
///
/// ```compile_fail,E0308
/// fn accepts_only_static(reference: &'static str) -> usize {
///     reference.len()
/// }
///
/// let looser: fn(&str) -> usize = accepts_only_static;  // mismatched types
/// ```
pub fn the_reverse_direction_is_rejected() {}

// ---------------------------------------------------------------------------
// Listing 1-11: the case that needs two lifetimes
// ---------------------------------------------------------------------------

/// Listing 1-11: *A type that needs to be generic over multiple lifetimes.*
///
/// Two lifetimes look redundant here — there are no methods at all — but
/// collapsing them into one makes the demo fail to compile. The field is a
/// `&'a mut &'b str`: `'a` is the borrow of `s`, `'b` is the lifetime of the
/// `&str` stored *in* `s`.
///
/// Attempting to access `s` afterwards requires shortening the mutable borrow.
/// With two lifetimes, `'a` can end at that point while `'b` stays `'static`.
/// With one lifetime the compiler would also have to shorten `'b` — but `'b` sits
/// behind a `&mut`, which is invariant, so it cannot be shortened. The borrow of
/// `s` therefore stays live, and the later access is rejected.
///
/// ```compile_fail,E0502
/// struct MutStr<'a> {
///     s: &'a mut &'a str,
/// }
///
/// let mut s = "hello";
/// *MutStr { s: &mut s }.s = "world";
/// println!("{}", s);   // cannot borrow `s` as immutable, it is still mutably borrowed
/// ```
pub struct MutStr<'a, 'b> {
    pub s: &'a mut &'b str,
}

pub fn listing_1_11_two_lifetimes_are_needed() {
    let mut s = "hello";

    // `'a` is the borrow of `s`; `'b` is inferred as `'static` from `"hello"`.
    *MutStr { s: &mut s }.s = "world";

    assert_eq!(s, "world");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn covariance_in_the_lifetime_runs() {
        covariance_in_the_lifetime();
    }

    #[test]
    fn covariance_in_the_type_runs() {
        covariance_in_the_type();
    }

    #[test]
    fn invariance_blocks_writing_a_shorter_lifetime_in_runs() {
        invariance_blocks_writing_a_shorter_lifetime_in();
    }

    #[test]
    fn contravariance_in_function_arguments_runs() {
        contravariance_in_function_arguments();
    }

    #[test]
    fn listing_1_11_two_lifetimes_are_needed_runs() {
        listing_1_11_two_lifetimes_are_needed();
    }
}
