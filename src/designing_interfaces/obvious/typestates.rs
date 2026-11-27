//! # Obvious — Typestates (Listing 3-2)
//!
//! > A closely related technique is to use zero-sized types to indicate that a
//! > particular fact is true about an instance of a type. Consider, for instance,
//! > a type called `Rocket` that represents the state of a real rocket. Some
//! > operations (methods) on `Rocket` should be available no matter what state the
//! > rocket is in, but some make sense only in particular situations. It is, for
//! > example, impossible to launch a rocket if it has already been launched.
//! > Similarly, it should probably not be possible to separate the fuel tank if
//! > the rocket has not yet launched. We could model these as enum variants, but
//! > then all the methods would be available at every stage, and we'd need to
//! > introduce possible panics.
//!
//! Listing 3-2 is implemented below as [`Rocket`]. Its counterpart — the enum
//! with runtime panics — is [`EnumRocket`], so the two costs can be compared
//! directly:
//!
//! | | [`Rocket<Stage>`] | [`EnumRocket`] |
//! | --- | --- | --- |
//! | wrong stage | `E0599`, no such method | compiles, panics at run time |
//! | size | zero: `PhantomData` is erased | at least one discriminant byte |
//! | stage is | a type parameter | a field anyone can read |
//!
//! > You'll notice that with the interface designed this way, it's simply not
//! > possible for the user to call a method at the wrong time—we have encoded the
//! > usage rules in the types themselves, and made illegal states
//! > _unrepresentable_.

use std::marker::PhantomData;
use std::mem::size_of;
use std::panic::{AssertUnwindSafe, catch_unwind};

// ---------------------------------------------------------------------------
// The stage markers
// ---------------------------------------------------------------------------

/// The rocket is on the pad: it can be launched, but not steered.
pub struct Grounded;

/// The rocket is in flight: it can be steered, but not launched again.
pub struct Launched;

/// What colour a rocket is painted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    /// The only colour this toy rocket comes in.
    pub const BLACK: Color = Color(0, 0, 0);
}

/// What a rocket weighs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Kilograms(pub f64);

impl Kilograms {
    /// An empty rocket.
    pub const ZERO: Kilograms = Kilograms(0.0);
}

// ---------------------------------------------------------------------------
// Listing 3-2
// ---------------------------------------------------------------------------

/// > We introduce unit types to represent each stage of the rocket. We don't
/// > actually need to store the stage—only the meta-information it provides—so
/// > we store it behind a `PhantomData` to guarantee that it is eliminated at
/// > compile time. Then, we write implementation blocks for `Rocket` only when it
/// > holds a particular type parameter.
///
/// The default type parameter is what lets `Rocket` mean `Rocket<Grounded>`, so a
/// rocket starts out on the ground whether or not the caller spells it:
///
/// ```
/// use rust_for_rustaceans::designing_interfaces::obvious::typestates::{Grounded, Rocket};
///
/// let short: Rocket = Rocket::default();
/// let explicit: Rocket<Grounded> = short;
/// assert_eq!(explicit.color(), rust_for_rustaceans::designing_interfaces::obvious::typestates::Color::BLACK);
/// ```
///
/// A launched rocket cannot be constructed the same way — the `Default` impl
/// exists only for the grounded stage:
///
/// ```compile_fail,E0599
/// use rust_for_rustaceans::designing_interfaces::obvious::typestates::{Launched, Rocket};
///
/// // no associated function or constant named `default` found
/// let _ = Rocket::<Launched>::default();
/// ```
pub struct Rocket<Stage = Grounded> {
    /// The stage is not stored, only declared: `PhantomData` has no fields and
    /// `size_of::<Rocket<Stage>>()` is zero for every stage.
    stage: PhantomData<Stage>,
}

/// > You can construct a rocket only on the ground (for now)
impl Default for Rocket<Grounded> {
    fn default() -> Self {
        Self { stage: PhantomData }
    }
}

/// > you can launch it only from the ground
impl Rocket<Grounded> {
    /// Consumes the grounded rocket and returns it in flight.
    ///
    /// The signature is the whole guarantee: `launch` takes `self` and exists
    /// only on `Rocket<Grounded>`, so a second launch is not a runtime error —
    /// there is no `launch` method on a `Rocket<Launched>` to call:
    ///
    /// ```compile_fail,E0599
    /// use rust_for_rustaceans::designing_interfaces::obvious::typestates::Rocket;
    ///
    /// let grounded = Rocket::default();
    /// // no method named `launch` found for struct `Rocket<Launched>`
    /// let _ = grounded.launch().launch();
    /// ```
    pub fn launch(self) -> Rocket<Launched> {
        Rocket { stage: PhantomData }
    }
}

/// > Only when the rocket has been launched can you control its velocity
impl Rocket<Launched> {
    /// Available only in flight. On the ground this method does not exist:
    ///
    /// ```compile_fail,E0599
    /// use rust_for_rustaceans::designing_interfaces::obvious::typestates::Rocket;
    ///
    /// let mut grounded = Rocket::default();
    /// // no method named `accelerate` found for struct `Rocket`
    /// grounded.accelerate();
    /// ```
    pub fn accelerate(&mut self) {}

    /// Available only in flight, for the same reason.
    pub fn decelerate(&mut self) {}
}

/// > There are some things you can always do with the rocket, no matter what
/// > state it is in, and those we place in a generic implementation block
impl<Stage> Rocket<Stage> {
    pub fn color(&self) -> Color {
        Color::BLACK
    }

    pub fn weight(&self) -> Kilograms {
        Kilograms::ZERO
    }
}

// ---------------------------------------------------------------------------
// The alternative the book rejects: an enum and a panic
// ---------------------------------------------------------------------------

/// Which stage the rocket is in, stored as data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Grounded,
    Launched,
}

/// The same state machine with the stage in a field: every method is available
/// at every stage, so the mistakes the type system refused above are now runtime
/// panics.
#[derive(Debug)]
pub struct EnumRocket {
    stage: Stage,
    velocity: i32,
}

impl EnumRocket {
    pub fn grounded() -> Self {
        Self {
            stage: Stage::Grounded,
            velocity: 0,
        }
    }

    /// # Panics
    ///
    /// Panics if the rocket has already been launched.
    pub fn launch(&mut self) {
        assert!(self.stage == Stage::Grounded, "already launched");
        self.stage = Stage::Launched;
    }

    /// # Panics
    ///
    /// Panics unless the rocket has been launched.
    pub fn accelerate(&mut self) {
        assert!(
            self.stage == Stage::Launched,
            "cannot accelerate before launch"
        );
        self.velocity += 1;
    }
}

// ---------------------------------------------------------------------------
// The two, side by side
// ---------------------------------------------------------------------------

/// > we can introduce a generic parameter on `Rocket` , `Stage` , and use it to
/// > restrict what methods are available when.
///
/// The legal sequence, with the cost of the stage visible in the size: the
/// typestate rocket is zero bytes, the enum rocket is not.
pub fn listing_3_2_the_stage_lives_in_the_type() {
    let grounded: Rocket<Grounded> = Rocket::default();

    // Methods from the generic impl block are available at every stage.
    assert_eq!(grounded.color(), Color::BLACK);
    assert_eq!(grounded.weight(), Kilograms::ZERO);

    // The stage costs nothing: only the meta-information exists.
    assert_eq!(size_of::<Rocket<Grounded>>(), 0);
    assert_eq!(size_of::<Rocket<Launched>>(), 0);

    let mut launched: Rocket<Launched> = grounded.launch();
    assert_eq!(launched.color(), Color::BLACK);
    launched.accelerate();
    launched.decelerate();
}

/// > We could model these as enum variants, but then all the methods would be
/// > available at every stage, and we'd need to introduce possible panics.
///
/// Both mistakes that were `E0599` above compile here, and both panic. They are
/// caught rather than left to abort the test, which is also the assertion: the
/// only thing standing between the caller and the panic is the caller.
pub fn an_enum_makes_the_same_mistakes_runtime_errors() {
    let mut rocket = EnumRocket::grounded();
    let accelerated_early = catch_unwind(AssertUnwindSafe(|| rocket.accelerate()));
    assert!(accelerated_early.is_err());

    let launched_twice = catch_unwind(AssertUnwindSafe(|| {
        let mut rocket = EnumRocket::grounded();
        rocket.launch();
        rocket.launch();
    }));
    assert!(launched_twice.is_err());

    // And the stage is data, so it occupies space the typestate version did not.
    assert!(size_of::<EnumRocket>() > 0);
    assert_eq!(size_of::<Rocket<Grounded>>(), 0);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn listing_3_2_the_stage_lives_in_the_type_runs() {
        listing_3_2_the_stage_lives_in_the_type();
    }

    #[test]
    fn an_enum_makes_the_same_mistakes_runtime_errors_runs() {
        an_enum_makes_the_same_mistakes_runtime_errors();
    }
}
