#![cfg_attr(not(test), no_std)]

//! Create a trusted carrier with a new lifetime that is guaranteed to be
//! unique among other trusted carriers. When you call [`guard!`] to make a
//! unique lifetime, the macro creates a [`Guard`] to hold it. This guard can be
//! converted `into` an [`Id`], which can be stored in structures to uniquely
//! "brand" them. A different invocation of the macro will produce a new
//! lifetime that cannot be unified. The only way to construct these types is
//! with [`guard!`] or `unsafe` code.
//!
//! ```rust
//! use generativity::{Id, guard};
//! struct Struct<'id>(Id<'id>);
//! let a = guard!();
//! Struct(a.into());
//! ```
//!
//! This is the concept of "generative" lifetime brands. `Guard` and `Id` are
//! [invariant](https://doc.rust-lang.org/nomicon/subtyping.html#variance) over
//! their lifetime parameter, meaning that it is never valid to substitute or
//! otherwise coerce `Id<'a>` into `Id<'b>`, for *any* concrete `'a` or `'b`,
//! *including* the `'static` lifetime.
//!
//! Any invariant lifetime can be "trusted" to carry a brand, so long as they
//! are known to be restricted to carrying a brand, and haven't been derived
//! from some untrusted lifetime (or are completely unbound). When using this
//! library, it is recommended to always use `Id<'id>` to carry the brand, as
//! this reduces the risk of accidentally trusting an untrusted lifetime.
//! Importantly, non-invariant lifetimes *cannot* be trusted, as the variance
//! allows lifetimes to be contracted to match and copy the brand lifetime.
//!
//! To achieve lifetime invariance without `Id`, there are two standard ways:
//! `PhantomData<&'a mut &'a ()>` and `PhantomData<fn(&'a ()) -> &'a ()>`. The
//! former works because `&mut T` is invariant over `T`, and the latter works
//! because `fn(T)` is *contra*variant over `T` and `fn() -> T` is *co*variant
//! over `T`, which combines to *in*variance. Both are equivalent in this case
//! with `T = ()`, but `fn(T) -> T` is generally preferred if the only purpose
//! is to indicate invariance, as function pointers are a perfect cover for all
//! auto traits (e.g. `Send`, `Sync`, `Unpin`, `UnwindSafe`, etc.) and thus
//! only indicates invariance, whereas `&mut T` can carry further implication
//! of "by example" use of `PhantomData`.

use core::fmt;
use core::marker::PhantomData;

/// A phantomdata-like type taking a single invariant lifetime.
///
/// Used to manipulate and store the unique invariant lifetime obtained from
/// [`Guard`]. Use `guard.into()` to create a new `Id`.
///
/// Holding `Id<'id>` indicates that the lifetime `'id` is a trusted brand.
/// `'id` will not unify with another trusted brand lifetime unless it comes
/// from the same original brand (i.e. the same invocation of [`guard!`]).
#[repr(transparent)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Id<'id> {
    phantom: PhantomData<fn(&'id ()) -> &'id ()>,
}

impl<'id> Id<'id> {
    /// Construct an `Id` with an unbounded lifetime.
    ///
    /// You should not need to use this function; use [`guard!`] instead.
    ///
    /// # Safety
    ///
    /// `Id` holds an invariant lifetime that must be derived from a generative
    /// brand. Using this function directly is the "I know what I'm doing"
    /// button; restrict the lifetime to a known brand immediately to avoid
    /// introducing potential unsoundness.
    pub unsafe fn new() -> Self {
        Id {
            phantom: PhantomData,
        }
    }
}

impl<'id> fmt::Debug for Id<'id> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("#[invariant] 'id").finish()
    }
}

impl<'id> From<Guard<'id>> for Id<'id> {
    fn from(guard: Guard<'id>) -> Self {
        guard.id
    }
}

/// An invariant lifetime phantomdata-alike that is guaranteed to be unique
/// with respect to other trusted invariant lifetimes.
///
/// In effect, this means that `'id` is a "generative brand". Use [`guard`]
/// to obtain a new `Guard`.
#[repr(transparent)]
#[derive(Eq, PartialEq)]
pub struct Guard<'id> {
    #[allow(unused)]
    id: Id<'id>,
}

impl<'id> Guard<'id> {
    /// Construct a `Guard` with an unbound lifetime.
    ///
    /// You should not need to use this function; use [`guard!`] instead.
    ///
    /// # Safety
    ///
    /// `Guard` holds an invariant lifetime that must be an unused generative
    /// brand. Using this function directly is the "I know what I'm doing"
    /// button; restrict the lifetime to a known brand immediately to avoid
    /// introducing potential unsoundness.
    pub unsafe fn new(id: Id<'id>) -> Guard<'id> {
        Guard { id }
    }
}

impl<'id> fmt::Debug for Guard<'id> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("#[unique] 'id").finish()
    }
}

#[doc(hidden)]
/// NOT STABLE PUBLIC API. Used by the expansion of [`guard!`].
pub struct LifetimeBrand<'id> {
    phantom: PhantomData<&'id Id<'id>>,
}

impl<'id> Drop for LifetimeBrand<'id> {
    #[inline(always)]
    fn drop(&mut self) {
        // This impl purposefully left blank. The presence of a Drop impl gives
        // the `make_guard` type drop glue, dropping it at the end of scope.
        // Importantly, this ensures that the compiler has to consider `'id`
        // live at the point that this type is dropped, because this impl could
        // potentially use data borrowed that lifetime. #[inline(always)] just
        // serves to make it easier to optimize out the noop function call.
    }
}

#[doc(hidden)]
/// NOT STABLE PUBLIC API. Used by the expansion of [`guard!`].
impl<'id> LifetimeBrand<'id> {
    #[doc(hidden)]
    #[inline(always)]
    /// NOT STABLE PUBLIC API. Used by the expansion of [`guard!`].
    pub unsafe fn new(_: &'id Id<'id>) -> LifetimeBrand<'id> {
        // This function serves to entangle the `'id` lifetime, making it into
        // a proper lifetime brand. The `'id` region may open at any point, but
        // it must end in-between the drop timing of this `LifetimeBrand` and
        // the `Id` binding used to create it.
        LifetimeBrand {
            phantom: PhantomData,
        }
    }
}

/// Create a `Guard` with a unique invariant lifetime (with respect to other
/// trusted/invariant lifetime brands).
///
/// An optional identifier can be provided so that local errors can point out
/// both conflicting brands if they are confused.
///
/// Multiple `guard!`ed lifetimes will always fail to unify:
///
/// ```rust,compile_fail,E0716
/// # // trybuild ui test tests/ui/crossed_streams-expr.rs
/// # #![feature(super_let)]
/// # use generativity::guard;
/// let a = guard!(a);
/// let b = guard!(b);
/// dbg!(a == b); // ERROR (here == is a static check)
/// ```
#[macro_export]
macro_rules! guard {
    () => {
        $crate::guard! { anonymous_generativity_brand }
    };
    ($name:ident) => {{
        // SAFETY: The lifetime given to `$name` is unique among trusted brands.
        // We know this because of how we carefully control drop timing here.
        // The branded lifetime's end is bound to be no later than when the
        // `branded_place` is invalidated at the end of scope, but also must be
        // no sooner than when `$name` is dropped, also at the end of scope.
        // Some other variant lifetime could be constrained to be equal to the
        // brand lifetime, but no other lifetime branded by `guard!` can,
        // as its brand lifetime has a distinct drop time from this one. QED
        super let branded_place = unsafe { $crate::Id::new() };
        #[allow(unused)] super let $name = unsafe { $crate::LifetimeBrand::new(&branded_place) };

        // The whole following `if let Some(_) = None {}` block has only one role: to handle
        // the case where follow-up code might diverge.
        // See https://github.com/CAD97/generativity/issues/15 for the history.
        //
        // This works due to the how the phases of rustc are currently organized:
        //  1. rustc always generates MIR if the block is syntactically reachable
        //    (meaning, ignoring types) so even if `x` here has type `!`, this
        //    will still generate MIR (including drop-check).
        //  2. rustc does type inference, which may resolve `x` to an
        //    uninhabited type, like `!`. However no MIR opts are done at this stage
        //  3. rustc does lifetime analysis to verify all references are used properly.
        //    invalid code which tries to confuse two different `Guard`s will
        //    FAIL to compile here, not warn, but hard error. Since it is a lifetime error
        //  4. we never reach MIR opts on this failure, so this code should work
        //
        // if rustc ever decides to do MIR opts between type check and lifetime check
        // then this pattern could break. But that's unlikely.
        //
        // This branch ensures that there is at least one place where the `LifetimeBrand`
        // is dropped. Which ensures that all `LifetimeBrand`s created will have unique lifetimes
        if let $crate::__private::Some(x) = $crate::__private::None {
            return x;
        } else {
            unsafe { $crate::Guard::new(branded_place) }
        }
    }};
}

/// Create a `Guard` with a unique invariant lifetime (with respect to other
/// trusted/invariant lifetime brands).
///
/// This is a statement macro version of [`guard!`] that works on Rust versions
/// before `#![feature(super_let)]`
#[macro_export]
macro_rules! make_guard {
    ($name:ident) => {
        // SAFETY: See guard! above.
        let branded_place = unsafe { $crate::Id::new() };
        // We could use $name instead of anonymous_generativity_brand, but this
        // leads to confusion of whether the drop timing note is about this or
        // the created Guard value.
        #[allow(unused)]
        let lifetime_brand = unsafe { $crate::LifetimeBrand::new(&branded_place) };
        let $name = unsafe { $crate::Guard::new(branded_place) };

        if let $crate::__private::Some(x) = $crate::__private::None {
            return x;
        }
    };
}

#[doc(hidden)]
/// NOT STABLE PUBLIC API. Used by the expansion of [`guard!`].
pub mod __private {
    pub use {None, Some};
}

#[cfg(test)]
mod test {
    use super::*;
    use std::panic::{RefUnwindSafe, UnwindSafe};

    #[test]
    fn dont_error_in_general() {
        make_guard!(a);
        make_guard!(b);
        assert_eq!(a, a);
        assert_eq!(b, b);
    }

    #[test]
    fn test_oibits() {
        fn assert_oibits<T>(_: &T)
        where
            // unstable oibits: UnsafeUnpin + Freeze
            T: Send + Sync + Unpin + UnwindSafe + RefUnwindSafe,
        {
        }

        make_guard!(guard);
        assert_oibits(&guard);
        let id: Id<'_> = guard.into();
        assert_oibits(&id);

        // const compatible (e.g. const_refs_to_cell, const destructor)
        const fn _const_id(_: Id<'_>) {}
        const fn _const_ref_id(_: &'_ Id<'_>) {}
        const fn _const_guard(_: Guard<'_>) {}
        const fn _const_ref_guard(_: &'_ Guard<'_>) {}
    }
}
