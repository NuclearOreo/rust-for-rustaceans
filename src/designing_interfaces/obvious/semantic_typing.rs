//! # Obvious — Semantic Typing
//!
//! > The type system is an excellent tool to ensure that your interfaces are
//! > obvious, self-documenting, and misuse-resistant.
//!
//! > The first of these is _semantic typing_ , in which you add types to
//! > represent the _meaning_ of a value, not just its primitive type.
//!
//! Four uses of the idea, in the book's order:
//!
//! | Primitive | Semantic type | Mistake it removes |
//! | --- | --- | --- |
//! | `bool, bool, bool` | one two-variant enum per flag | passing `DryRun::Yes` where `Overwrite::No` goes (`E0308`) |
//! | `f64` | `Meters` / `Feet` newtypes | adding a distance in the wrong unit (`E0308`) |
//! | `usize` index | `Slot`, constructible only by `Store::insert` | forging an index that was never handed out (`E0603`) |
//! | `(bool, "where")` | `Target::Nowhere` / `Target::At(..)` | a "go nowhere" request that still carries a destination (`E0618`) |
//!
//! > The classic example here is for Booleans: if your function takes three
//! > `bool` arguments, chances are some user will mess up the order of the values
//! > and realize it only after something has gone terribly wrong. If, on the
//! > other hand, it takes three arguments of distinct two-variant enum types, the
//! > user cannot get the order wrong without the compiler yelling at them.
//!
//! > You can apply semantic typing beyond Booleans as well. For example, a
//! > newtype around a numeric type may provide a unit for the contained value, or
//! > it could constrain raw pointer arguments to only those that have been
//! > returned by another method.
//!
//! > This notion extends to many other domains as well; if your function ignores
//! > a pointer argument unless a given Boolean argument is true, it's better to
//! > combine the two arguments instead. With an enum type with one variant for
//! > `false` (and no pointer) and one variant for `true` that holds a pointer,
//! > neither the caller nor the implementer can misunderstand the relationship
//! > between the two.

use std::ops::Add;

// ---------------------------------------------------------------------------
// Three bools, or three enums
// ---------------------------------------------------------------------------

/// One two-variant enum per flag: the *meaning* of the value is the type.
pub enum Overwrite {
    Yes,
    No,
}

pub enum DryRun {
    Yes,
    No,
}

pub enum Verbose {
    Yes,
    No,
}

/// The version the book warns about: three `bool`s whose order nobody can check.
pub fn publish_bools(
    name: &str,
    overwrite: bool,
    dry_run: bool,
    verbose: bool,
) -> Vec<&'static str> {
    let mut steps = Vec::new();
    if dry_run {
        steps.push("would write");
    } else {
        steps.push("wrote");
        if overwrite {
            steps.push("replaced the old copy");
        }
    }
    if verbose {
        steps.push("reported every step");
    }
    let _ = name;
    steps
}

/// The version where the argument order is checked by the compiler.
///
/// A caller who swaps two of the flags does not get a wrong result — they get an
/// error, because `DryRun` and `Overwrite` are different types:
///
/// ```compile_fail,E0308
/// use rust_for_rustaceans::designing_interfaces::obvious::semantic_typing::{
///     publish, DryRun, Overwrite, Verbose,
/// };
///
/// // arguments to this function are incorrect:
/// // expected `Overwrite`, found `DryRun`
/// let _ = publish("report.txt", DryRun::Yes, Overwrite::No, Verbose::No);
/// ```
pub fn publish(
    name: &str,
    overwrite: Overwrite,
    dry_run: DryRun,
    verbose: Verbose,
) -> Vec<&'static str> {
    publish_bools(
        name,
        matches!(overwrite, Overwrite::Yes),
        matches!(dry_run, DryRun::Yes),
        matches!(verbose, Verbose::Yes),
    )
}

/// > the user cannot get the order wrong without the compiler yelling at them …
/// > nor will passing `Overwrite::No` as the `dry_run` argument.
///
/// The bool version accepts both readings of the same call site and quietly picks
/// one; the enum version makes the two readings different programs. That is the
/// whole of the difference — nothing about the runtime work changed.
pub fn three_bools_accept_a_mistake_the_enums_refuse() {
    // The caller meant "replace the old copy and do it now".
    // They wrote the arguments in the other order, and it still compiled.
    let mistaken = publish_bools("report.txt", false, true, false);
    assert_eq!(mistaken, ["would write"]);

    // The same call site in the enum version does not compile at all; what
    // compiles is the version that says what it means.
    let intended = publish("report.txt", Overwrite::Yes, DryRun::No, Verbose::No);
    assert_eq!(intended, ["wrote", "replaced the old copy"]);
}

// ---------------------------------------------------------------------------
// A newtype gives the number a unit
// ---------------------------------------------------------------------------

/// A distance in metres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Meters(pub f64);

/// A distance in feet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Feet(pub f64);

impl Feet {
    /// The conversion the type system forces the caller to make explicit.
    pub fn to_meters(self) -> Meters {
        Meters(self.0 * 0.3048)
    }
}

impl Meters {
    pub fn value(self) -> f64 {
        self.0
    }
}

impl Add for Meters {
    type Output = Meters;

    fn add(self, other: Meters) -> Meters {
        Meters(self.0 + other.0)
    }
}

/// > a newtype around a numeric type may provide a unit for the contained value
///
/// Metres can be added to metres, and feet to metres only after an explicit
/// conversion — the accidental mixed-unit sum is a type error:
///
/// ```compile_fail,E0308
/// use rust_for_rustaceans::designing_interfaces::obvious::semantic_typing::{Feet, Meters};
///
/// // cannot add `Feet` to `Meters`: mismatched types
/// let total = Meters(1.0) + Feet(2.0);
/// ```
pub fn a_newtype_gives_the_number_a_unit() {
    let one_metre = Meters(1.0);
    let three_feet = Feet(3.280_84);

    let total = one_metre + three_feet.to_meters();
    assert!((total.value() - 2.0).abs() < 1e-6);

    // The raw numbers would have added up too — to the wrong answer.
    assert!((1.0_f64 + 3.280_84 - 2.0).abs() > 2.0);
}

// ---------------------------------------------------------------------------
// A capability token: the only way to get one is to be given one
// ---------------------------------------------------------------------------

/// A position in a [`Store`]. The field is private, so no other crate can write
/// `Slot(0)` and pretend it was handed one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot(usize);

/// A list whose positions can only be used if they came from [`Store::insert`].
#[derive(Debug, Default)]
pub struct Store {
    values: Vec<&'static str>,
}

impl Store {
    /// Hands back the token for the inserted value.
    pub fn insert(&mut self, value: &'static str) -> Slot {
        self.values.push(value);
        Slot(self.values.len() - 1)
    }

    /// Accepts nothing but a [`Slot`].
    ///
    /// # Panics
    ///
    /// Panics if `slot` came from a different `Store` with fewer entries — the
    /// token is unforgeable, but it does not carry its store's identity.
    pub fn get(&self, slot: Slot) -> &'static str {
        self.values[slot.0]
    }
}

/// > it could constrain raw pointer arguments to only those that have been
/// > returned by another method
///
/// `Slot` has a private field, so the constructor is not reachable from another
/// crate — a doctest is another crate, so this is the real privacy boundary:
///
/// ```compile_fail,E0603
/// use rust_for_rustaceans::designing_interfaces::obvious::semantic_typing::Slot;
///
/// // tuple struct constructor `Slot` is private
/// let forged = Slot(0);
/// ```
///
/// The token is unforgeable but not provenance-checked: a `Slot` from one store
/// is accepted by another, which is what a lifetime or a marker type would be
/// needed to prevent.
pub fn a_capability_token_is_unforgeable_but_not_provenanced() {
    let mut store = Store::default();
    let alpha = store.insert("alpha");
    let beta = store.insert("beta");

    assert_eq!(store.get(alpha), "alpha");
    assert_eq!(store.get(beta), "beta");

    // The same token against a different store: accepted, because `Slot` says
    // "an index I gave out", not "an index this store gave out".
    let other = {
        let mut other = Store::default();
        other.insert("gamma");
        other
    };
    assert_eq!(other.get(alpha), "gamma");
}

// ---------------------------------------------------------------------------
// A bool and a pointer that disagree
// ---------------------------------------------------------------------------

/// The two facts that belong together, as one type: nowhere to go, or somewhere
/// to go with the place to go.
#[derive(Debug, PartialEq, Eq)]
pub enum Target {
    Nowhere,
    At(&'static str),
}

/// The version the book replaces: a flag and a destination that can disagree.
pub fn travel_with_bools(at: bool, where_to: &'static str) -> String {
    if at {
        format!("moved to {where_to}")
    } else {
        "stayed put".to_owned()
    }
}

/// The version where the relationship cannot be misread.
pub fn travel_to(target: Target) -> String {
    match target {
        Target::Nowhere => "stayed put".to_owned(),
        Target::At(where_to) => format!("moved to {where_to}"),
    }
}

/// > With an enum type with one variant for `false` (and no pointer) and one
/// > variant for `true` that holds a pointer, neither the caller nor the
/// > implementer can misunderstand the relationship between the two.
///
/// The `Nowhere` variant has no payload, so attaching one is a type error rather
/// than an ignored argument:
///
/// ```compile_fail,E0618
/// use rust_for_rustaceans::designing_interfaces::obvious::semantic_typing::Target;
///
/// // expected function, found `Target`: `Nowhere` takes no destination
/// let _ = Target::Nowhere("somewhere");
/// ```
pub fn a_flag_and_a_pointer_can_disagree_an_enum_cannot() {
    // `at = false` but a destination was passed: silently ignored.
    assert_eq!(travel_with_bools(false, "somewhere"), "stayed put");

    // Neither variant can hold both facts, so neither reading is possible.
    assert_eq!(travel_to(Target::Nowhere), "stayed put");
    assert_eq!(travel_to(Target::At("somewhere")), "moved to somewhere");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn three_bools_accept_a_mistake_the_enums_refuse_runs() {
        three_bools_accept_a_mistake_the_enums_refuse();
    }

    #[test]
    fn a_newtype_gives_the_number_a_unit_runs() {
        a_newtype_gives_the_number_a_unit();
    }

    #[test]
    fn a_capability_token_is_unforgeable_but_not_provenanced_runs() {
        a_capability_token_is_unforgeable_but_not_provenanced();
    }

    #[test]
    fn a_flag_and_a_pointer_can_disagree_an_enum_cannot_runs() {
        a_flag_and_a_pointer_can_disagree_an_enum_cannot();
    }
}
