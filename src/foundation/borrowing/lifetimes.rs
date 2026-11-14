//! # Lifetimes
//!
//! The common teaching — "a lifetime begins when you take a reference and ends
//! when the variable goes out of scope" — is "often correct, and usually useful,
//! but the reality is a little more complex."
//!
//! > A *lifetime* is really a name for a region of code that some reference must
//! > be valid for. While a lifetime will frequently coincide with a scope, it
//! > does not have to.
//!
//! Two ways they diverge are shown here: a lifetime that stops short of the
//! enclosing scope (Listing 1-8), and one with *holes* in it — "intermittently
//! invalid between where it starts and where it ultimately ends" (Listing 1-9).
//!
//! The mechanism is the borrow checker: "whenever a reference with some lifetime
//! `'a` is used, the borrow checker checks that `'a` is still *alive*. It does
//! this by tracing the path back to where `'a` starts ... and checking that there
//! are no conflicting uses along that path." That is the flow model from
//! [`memory::variables`](super::super::memory::variables), applied to references.

/// A borrow ends at its last *use*, not at the end of the enclosing scope. The
/// borrow checker works on flows, so the lifetime of `r` stops after its final
/// read, leaving `x` free to be written afterwards.
///
/// ```compile_fail,E0506
/// let mut x = Box::new(42);
/// let r = &x;
/// *x = 84;          // cannot assign to `x` because it is borrowed
/// println!("{}", r);
/// ```
pub fn a_lifetime_ends_at_its_last_use() {
    let mut x = Box::new(42);

    let r = &x;
    assert_eq!(**r, 42); // last use of `r`

    *x = 84; // legal: no flow from `r` reaches here
    assert_eq!(*x, 84);
}

/// Listing 1-8: *Lifetimes do not need to be contiguous.*
///
/// `r` is still in scope in the `if` branch, yet `*x = 84` there is accepted.
/// The borrow's region does not extend into that branch, because there is no use
/// of `r` along it. The checker walks back from each use separately and finds no
/// conflicting use on either path.
///
/// Adding a use of `r` *after* the branch would make the two paths meet again and
/// the lifetime contiguous — and the code would then be rejected.
pub fn listing_1_8_lifetimes_need_not_be_contiguous(take_write_branch: bool) {
    let mut x = Box::new(42);
    let r = &x;

    if take_write_branch {
        *x = 84; // accepted: no use of `r` on this path
    } else {
        assert_eq!(**r, 42); // `r` is used only on this path
    }

    assert_eq!(*x, if take_write_branch { 84 } else { 42 });
}

/// Listing 1-9: *Lifetimes can have holes.*
///
/// Moving out of `x` ends the borrow of `x`; taking a new reference to `x`
/// restarts it. Between those two points the lifetime is not valid at all, so it
/// is a region with a gap in it rather than one continuous span.
///
/// Viewed through the flow model this is unsurprising: when `x` is moved, `z`
/// stops existing, and the later `z = &x` creates a new variable that happens to
/// reuse the name.
// Clippy would rather write `*x = i`, which writes *through* the box and so does
// not move out of `x` — the borrow would stay live and the listing would stop
// demonstrating a lifetime ending. The fresh allocation is the point.
#[allow(clippy::replace_box)]
pub fn listing_1_9_lifetimes_can_have_holes() {
    let mut x = Box::new(42);
    let mut z = &x; // 'a starts here

    for i in 0..3 {
        // First pass sees 42; later passes see the previous pass's value.
        assert_eq!(**z, if i == 0 { 42 } else { i - 1 });

        x = Box::new(i); // move out: 'a ends here
        z = &x; // new borrow: 'a restarts here
    }

    assert_eq!(**z, 2);
}

/// The borrow checker is conservative by design: "if it's unsure whether a borrow
/// is valid, it rejects it, as the consequences of allowing an invalid borrow
/// could be disastrous." That is why some legal programs need `unsafe` to state
/// a guarantee the checker cannot derive.
///
/// Two exclusive borrows at once is the clearest case it will always refuse:
///
/// ```compile_fail,E0499
/// let mut x = 1;
/// let a = &mut x;
/// let b = &mut x;   // cannot borrow `x` as mutable more than once at a time
/// *a += 1;          // `a` is still needed here, so the borrows really overlap
/// *b += 1;
/// ```
///
/// Note that without the two writes the code would compile: an unused borrow
/// ends immediately, so `a` would be dead before `b` began.
pub fn two_exclusive_borrows_are_impossible() {
    let mut x = 1;

    {
        let a = &mut x;
        *a += 1;
    } // `a` ends here

    {
        let b = &mut x; // so this one is fine
        *b += 1;
    }

    assert_eq!(x, 3);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn a_lifetime_ends_at_its_last_use_runs() {
        a_lifetime_ends_at_its_last_use();
    }

    #[test]
    fn listing_1_8_runs_on_both_paths() {
        listing_1_8_lifetimes_need_not_be_contiguous(true);
        listing_1_8_lifetimes_need_not_be_contiguous(false);
    }

    #[test]
    fn listing_1_9_lifetimes_can_have_holes_runs() {
        listing_1_9_lifetimes_can_have_holes();
    }

    #[test]
    fn two_exclusive_borrows_are_impossible_runs() {
        two_exclusive_borrows_are_impossible();
    }
}
