//! # Dropping, and drop order
//!
//! "When a value's owner no longer has use for it, it is the owner's
//! responsibility to do any necessary cleanup for that value by *dropping* it."
//!
//! Dropping is automatic — it happens when the owning variable goes out of
//! scope — and it is recursive: "types usually recursively drop values they
//! contain, so dropping a variable of a complex type may result in many values
//! being dropped." Ownership is what makes it safe: with exactly one owner,
//! "we cannot accidentally drop the same value multiple times." A reference does
//! not own its referent, so dropping a reference drops nothing.
//!
//! # The two ordering rules
//!
//! The book's **DROP ORDER** box gives rules that look inconsistent until you
//! see the reason:
//!
//! * **Variables** (including function parameters) drop in *reverse* order of
//!   declaration.
//! * **Nested values** — tuple elements, array elements, struct fields — drop in
//!   *source* order.
//!
//! The asymmetry is deliberate. A later variable may hold a reference *into* an
//! earlier one, so earlier values must outlive later ones: reverse order. But a
//! single value cannot currently contain a reference to itself, so nested values
//! are free to drop in the intuitive front-to-back order.
//!
//! Every claim below is asserted by observing real `Drop` calls, not assumed.

use std::sync::Arc;
use std::sync::Mutex;

/// A shared record of the order in which [`Tracked`] values were dropped.
#[derive(Clone, Default)]
struct DropLog(Arc<Mutex<Vec<&'static str>>>);

impl DropLog {
    fn record(&self, name: &'static str) {
        self.0.lock().unwrap().push(name);
    }

    fn observed(&self) -> Vec<&'static str> {
        self.0.lock().unwrap().clone()
    }
}

/// A value that reports its own destruction.
struct Tracked {
    name: &'static str,
    log: DropLog,
}

impl Tracked {
    fn new(name: &'static str, log: &DropLog) -> Self {
        Self {
            name,
            log: log.clone(),
        }
    }
}

impl Drop for Tracked {
    fn drop(&mut self) {
        self.log.record(self.name);
    }
}

// ---------------------------------------------------------------------------
// Dropping happens: automatically, recursively, exactly once
// ---------------------------------------------------------------------------

/// A compound value owns what it contains, so dropping it drops its contents.
/// Here a `Box` owns its heap value; dropping the box runs that value's `Drop`.
pub fn dropping_a_compound_value_drops_its_parts() {
    let log = DropLog::default();

    {
        let _boxed = Box::new(Tracked::new("inside a Box", &log));
        assert!(log.observed().is_empty()); // still alive
    } // the Box is dropped, which drops the value it owns

    assert_eq!(log.observed(), ["inside a Box"]);
}

/// The same is true of collections: pushing a value into a `Vec` moves ownership
/// into the collection, so dropping the `Vec` drops every element it owns.
pub fn dropping_a_vector_drops_every_element() {
    let log = DropLog::default();

    {
        let mut elements = Vec::with_capacity(2);
        for name in ["element 0", "element 1"] {
            elements.push(Tracked::new(name, &log)); // ownership moves into the Vec
        }
    } // dropping the Vec drops each element it owns

    assert_eq!(log.observed(), ["element 0", "element 1"]);
}

/// "A variable that holds a reference to another value does not own that other
/// value, so the value isn't dropped when the variable drops." The reference
/// going out of scope is a non-event; the referent's owner is still in charge.
pub fn references_do_not_own_their_referent() {
    let log = DropLog::default();

    {
        let owner = Tracked::new("owner", &log);

        {
            let _borrowed: &Tracked = &owner;
            assert!(log.observed().is_empty());
        } // the reference goes out of scope: nothing is dropped

        assert!(log.observed().is_empty());
    } // only now does the owner go out of scope

    assert_eq!(log.observed(), ["owner"]);
}

/// Moving a value moves the responsibility to clean it up. The value is dropped
/// at the end of the scope that currently owns it — not the one that created it.
pub fn a_moved_value_is_dropped_by_its_new_owner() {
    let log = DropLog::default();

    let value = Tracked::new("moved inward", &log);

    {
        let _moved = value; // ownership moves into this scope
    } // and is dropped here

    assert_eq!(log.observed(), ["moved inward"]);
}

/// `drop` takes ownership and so ends the value's life immediately, rather than
/// waiting for the enclosing scope to end.
pub fn drop_runs_cleanup_early() {
    let log = DropLog::default();

    let value = Tracked::new("dropped early", &log);
    drop(value); // consume it now

    assert_eq!(log.observed(), ["dropped early"]);
}

// ---------------------------------------------------------------------------
// Rule 1: variables drop in reverse order of declaration
// ---------------------------------------------------------------------------

pub fn variables_drop_in_reverse_order() {
    let log = DropLog::default();

    {
        let _first = Tracked::new("first", &log);
        let _second = Tracked::new("second", &log);
        let _third = Tracked::new("third", &log);
    }

    assert_eq!(log.observed(), ["third", "second", "first"]);
}

/// Function parameters are variables too, and follow the same reverse rule.
pub fn function_parameters_drop_in_reverse_order() {
    fn callee(_first: Tracked, _second: Tracked) {}

    let log = DropLog::default();

    callee(
        Tracked::new("parameter first", &log),
        Tracked::new("parameter second", &log),
    );

    assert_eq!(log.observed(), ["parameter second", "parameter first"]);
}

/// Why reverse order? Because a later variable may borrow from an earlier one,
/// and so must be destroyed first. Dropping forward would leave the borrower
/// pointing at freed memory — precisely what the ordering rule avoids.
pub fn later_variables_may_reference_earlier_ones() {
    {
        let text = String::from("owned by an earlier variable");
        let borrows = [&text]; // a *later* variable referring to an *earlier* one

        assert_eq!(borrows[0], "owned by an earlier variable");
    } // `borrows` is dropped before `text`, so the borrow never dangles
}

// ---------------------------------------------------------------------------
// Rule 2: nested values drop in source order
// ---------------------------------------------------------------------------

pub fn tuple_elements_drop_in_source_order() {
    let log = DropLog::default();

    {
        let _tuple = (
            Tracked::new("field 0", &log),
            Tracked::new("field 1", &log),
            Tracked::new("field 2", &log),
        );
    }

    assert_eq!(log.observed(), ["field 0", "field 1", "field 2"]);
}

pub fn array_elements_drop_in_index_order() {
    let log = DropLog::default();

    {
        let _array = [Tracked::new("index 0", &log), Tracked::new("index 1", &log)];
    }

    assert_eq!(log.observed(), ["index 0", "index 1"]);
}

pub fn struct_fields_drop_in_declaration_order() {
    // The fields exist only to be dropped.
    #[allow(dead_code)]
    struct Pair {
        first: Tracked,
        second: Tracked,
    }

    let log = DropLog::default();

    {
        let _pair = Pair {
            first: Tracked::new("first field", &log),
            second: Tracked::new("second field", &log),
        };
    }

    assert_eq!(log.observed(), ["first field", "second field"]);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn dropping_a_compound_value_drops_its_parts_runs() {
        dropping_a_compound_value_drops_its_parts();
    }

    #[test]
    fn dropping_a_vector_drops_every_element_runs() {
        dropping_a_vector_drops_every_element();
    }

    #[test]
    fn references_do_not_own_their_referent_runs() {
        references_do_not_own_their_referent();
    }

    #[test]
    fn a_moved_value_is_dropped_by_its_new_owner_runs() {
        a_moved_value_is_dropped_by_its_new_owner();
    }

    #[test]
    fn drop_runs_cleanup_early_runs() {
        drop_runs_cleanup_early();
    }

    #[test]
    fn variables_drop_in_reverse_order_runs() {
        variables_drop_in_reverse_order();
    }

    #[test]
    fn function_parameters_drop_in_reverse_order_runs() {
        function_parameters_drop_in_reverse_order();
    }

    #[test]
    fn later_variables_may_reference_earlier_ones_runs() {
        later_variables_may_reference_earlier_ones();
    }

    #[test]
    fn tuple_elements_drop_in_source_order_runs() {
        tuple_elements_drop_in_source_order();
    }

    #[test]
    fn array_elements_drop_in_index_order_runs() {
        array_elements_drop_in_index_order();
    }

    #[test]
    fn struct_fields_drop_in_declaration_order_runs() {
        struct_fields_drop_in_declaration_order();
    }
}
