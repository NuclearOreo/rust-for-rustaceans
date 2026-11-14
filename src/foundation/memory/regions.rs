//! # Memory Regions
//!
//! The three regions the book singles out, because each one dictates how long a
//! value can be reached:
//!
//! * The **stack** — scratch space for function calls, where each call gets a
//!   *frame* that is reclaimed on return.
//! * The **heap** — memory not tied to the call stack, living until it is
//!   explicitly deallocated.
//! * **Static memory** — whatever the compiled binary already contains, loaded
//!   when the program starts and alive until it exits.
//!
//! Which region you choose "has a significant impact on how you write your
//! code," and this file walks the consequences of each choice.

use std::mem::size_of;
use std::mem::size_of_val;
use std::ptr;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

// ---------------------------------------------------------------------------
// The stack
// ---------------------------------------------------------------------------

/// Each function call pushes a *frame* holding that call's locals and arguments.
/// Frames that are alive at the same time cannot overlap, which is why a local in
/// a callee has a different address from a local in its caller.
pub fn live_frames_occupy_distinct_memory() {
    fn callee_returns_address_of_its_local() -> usize {
        let inner = 0u8;
        ptr::addr_of!(inner) as usize
    }

    let outer = 0u8;
    let outer_addr = ptr::addr_of!(outer) as usize;
    let inner_addr = callee_returns_address_of_its_local();

    assert_ne!(outer_addr, inner_addr);
}

/// Returning a reference to a local is rejected because the frame it points into
/// is reclaimed when the function returns: "any reference to it must have a
/// lifetime that is at most as long as the lifetime of the frame."
///
/// ```compile_fail,E0515
/// fn dangling() -> &'static i32 {
///     let local = 42;
///     &local            // cannot return reference to local variable `local`
/// }
/// ```
///
/// The fix is to move the value *out* of the frame instead of pointing into it.
/// Ownership moves; the value ends up in the caller's frame.
#[allow(clippy::let_and_return)] // the named local *is* the point of the demo
pub fn move_the_value_out_of_the_frame() -> i32 {
    let local = 42;
    local // the value moves into the caller's frame; nothing is left behind
}

/// A stack frame's bytes "are not immediately wiped" when it is reclaimed — they
/// are simply not safe to use, since a later call may reuse the same memory or
/// the leftover bytes may hold values that were moved out. This is why Rust
/// makes raw pointers the only way to reach reclaimed stack memory, rather than
/// exposing a safe handle to it.
///
/// A caller-provided slot is the other way to keep a value alive past the call:
/// the caller leaves room on its own stack for the callee to write into.
pub fn return_by_writing_into_the_callers_slot() {
    fn fill(slot: &mut [u8; 4]) {
        *slot = [1, 2, 3, 4]; // written into the *caller's* frame
    }

    let mut buffer = [0u8; 4];
    let buffer_addr = ptr::addr_of!(buffer) as usize;

    fill(&mut buffer);

    assert_eq!(buffer, [1, 2, 3, 4]);
    assert_eq!(ptr::addr_of!(buffer) as usize, buffer_addr);
}

// ---------------------------------------------------------------------------
// The heap
// ---------------------------------------------------------------------------

/// `Box::new` puts the value on the heap and hands back a pointer to it. The
/// `Box` itself is only a pointer-sized slot on the stack, so *moving the box*
/// copies one pointer and leaves the heap value exactly where it was. That is
/// what makes heap values safe to hand outlive their creating frame.
pub fn moving_a_box_does_not_move_the_heap_value() {
    let boxed = Box::new(42u64);
    assert_eq!(size_of_val(&boxed), size_of::<usize>());

    let heap_before = &*boxed as *const u64;
    let moved = boxed; // move: the pointer is copied, ownership transfers
    let heap_after = &*moved as *const u64;

    assert_eq!(heap_before, heap_after); // same heap allocation
    assert_eq!(*moved, 42);
}

/// Because an allocation is not tied to the current call stack, it survives the
/// frame that created it. The returned pointer's lifetime is unconstrained: it
/// lasts as long as the program keeps it alive.
pub fn heap_value_outlives_the_creating_frame() {
    #[derive(Debug, PartialEq)]
    struct Pixel {
        r: u8,
        g: u8,
        b: u8,
    }

    fn make() -> Box<Pixel> {
        Box::new(Pixel {
            r: 255,
            g: 0,
            b: 127,
        })
    }

    // `make`'s frame is gone by the time this line runs; the pixel is not.
    let value = make();
    assert_eq!(
        *value,
        Pixel {
            r: 255,
            g: 0,
            b: 127
        }
    );
}

/// An allocation is freed when its owner is dropped — that is the whole point of
/// combining `Box` with Rust's ownership rules. Forgetting to deallocate would
/// *leak* memory instead, which is why `Drop` matters.
pub fn dropping_the_box_frees_the_allocation() {
    struct Tracked(Arc<AtomicUsize>);

    impl Drop for Tracked {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    let drops = Arc::new(AtomicUsize::new(0));
    {
        let boxed = Box::new(Tracked(Arc::clone(&drops)));
        assert_eq!(drops.load(Ordering::SeqCst), 0);

        drop(boxed); // frees the heap allocation and runs `Drop`
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }

    // Freed exactly once: no double free, no leak.
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

/// Configuration that the whole program should be able to read is a case where
/// leaking is deliberate: give up ownership with `Box::leak` and get a
/// `&'static mut T` back.
pub struct Settings {
    pub verbosity: u8,
}

/// The interesting part is what `Box::leak` proves about `'static`: this
/// reference is `'static` even though its bytes live on the *heap*, not in static
/// memory. `'static` means "valid for as long as the program likes," not
/// "located in static memory."
pub fn leak_to_get_a_static_reference() {
    let leaked: &'static mut Settings = Box::leak(Box::new(Settings { verbosity: 3 }));

    leaked.verbosity += 1;

    let as_static: &'static Settings = leaked;
    assert_eq!(as_static.verbosity, 4);
}

// ---------------------------------------------------------------------------
// Static memory
// ---------------------------------------------------------------------------

/// A `static` item is allocated when the program starts and never deallocated,
/// so it has exactly one address for the whole run.
pub static GREETING: &str = "Hello world";

pub fn a_static_has_one_address_for_the_whole_run() {
    let first = ptr::addr_of!(GREETING) as usize;
    let second = ptr::addr_of!(GREETING) as usize;

    assert_eq!(first, second);

    // A reference into a `static` is `'static` by definition, because the memory
    // is not deallocated until the program shuts down.
    let reference: &'static &'static str = &GREETING;
    assert_eq!(*reference, "Hello world");
}

/// `const` is *not* a place. A constant is "completely computed at compile time,"
/// and every use site is replaced with its value — so it has no storage and no
/// address of its own. `static` is the opposite: real memory, one address, shared
/// by all readers.
///
/// The giveaway is where each may appear: a `const` can be used in a constant
/// context such as an array length, while a `static` cannot, because reading a
/// `static` is a runtime operation.
pub const SIX: u8 = 6;

pub fn const_is_a_name_for_a_value() {
    // A constant context: requires an actual compile-time value, not a place.
    const ARRAY: [u8; SIX as usize] = [0; SIX as usize];

    assert_eq!(ARRAY.len(), 6);
    assert_eq!(SIX * 2, 12);
    assert_eq!(size_of_val(&SIX), 1);
}

/// The program's own code is static memory too — the text segment, normally
/// mapped read-only. A function pointer is simply an address in that segment,
/// which is why every reference to one function yields the same address.
fn marker() {}

pub fn functions_live_in_the_text_segment() {
    let first: fn() = marker;
    let second: fn() = marker;

    assert_eq!(first as usize, second as usize);
    assert!(first as usize != 0);
}

// ---------------------------------------------------------------------------
// `'static` as a bound
// ---------------------------------------------------------------------------

/// `'static` shows up far more often as a *bound* than as a lifetime on real
/// static memory. `T: 'static` requires that `T` be owned and self-sufficient:
/// it borrows nothing that could go away.
///
/// That is precisely why `thread::spawn` demands it — the new thread may outlive
/// the caller's frame, so it may not refer to anything stored on that stack.
pub fn spawn_requires_owned_data() {
    let owned = String::from("moved into the thread");

    let handle = std::thread::spawn(move || owned.len());

    assert_eq!(handle.join().unwrap(), 21);
}

/// Borrowing a local into a spawned thread is rejected, because the closure is
/// not `'static`:
///
/// ```compile_fail,E0597
/// let local = String::from("on the stack");
/// let borrowed = &local;
/// std::thread::spawn(move || borrowed.len());  // `local` does not live long enough
/// ```
///
/// The same rule applies to any `T: 'static` bound — a `&'static str` satisfies
/// it, a reference to a local does not.
///
/// ```compile_fail,E0597
/// fn requires_static<T: 'static>(value: T) -> T { value }
///
/// let local = String::from("on the stack");
/// requires_static(&local);
/// ```
pub fn static_bound_means_no_borrows() {
    fn requires_static<T: 'static>(value: T) -> T {
        value
    }

    // Owned data: self-sufficient, so it qualifies.
    let owned = requires_static(String::from("owned"));
    assert_eq!(owned.len(), 5);

    // A string literal is `&'static str`: the borrow is itself `'static`.
    let literal = requires_static("a literal");
    assert_eq!(literal.len(), 9);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn live_frames_occupy_distinct_memory_runs() {
        live_frames_occupy_distinct_memory();
    }

    #[test]
    fn move_the_value_out_of_the_frame_runs() {
        assert_eq!(move_the_value_out_of_the_frame(), 42);
    }

    #[test]
    fn return_by_writing_into_the_callers_slot_runs() {
        return_by_writing_into_the_callers_slot();
    }

    #[test]
    fn moving_a_box_does_not_move_the_heap_value_runs() {
        moving_a_box_does_not_move_the_heap_value();
    }

    #[test]
    fn heap_value_outlives_the_creating_frame_runs() {
        heap_value_outlives_the_creating_frame();
    }

    #[test]
    fn dropping_the_box_frees_the_allocation_runs() {
        dropping_the_box_frees_the_allocation();
    }

    #[test]
    fn leak_to_get_a_static_reference_runs() {
        leak_to_get_a_static_reference();
    }

    #[test]
    fn a_static_has_one_address_for_the_whole_run_runs() {
        a_static_has_one_address_for_the_whole_run();
    }

    #[test]
    fn const_is_a_name_for_a_value_runs() {
        const_is_a_name_for_a_value();
    }

    #[test]
    fn functions_live_in_the_text_segment_runs() {
        functions_live_in_the_text_segment();
    }

    #[test]
    fn spawn_requires_owned_data_runs() {
        spawn_requires_owned_data();
    }

    #[test]
    fn static_bound_means_no_borrows_runs() {
        static_bound_means_no_borrows();
    }
}
