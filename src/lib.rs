#![cfg_attr(not(test), no_std)]

//! Create a trusted carrier with a new lifetime that is guaranteed to be
//! unique among other trusted carriers. When you call [`make_guard!`] to make a
//! unique lifetime, the macro creates a [`Guard`] to hold it. This guard can be
//! converted `into` an [`Id`], which can be stored in structures to uniquely
//! "brand" them. A different invocation of the macro will produce a new
//! lifetime that cannot be unified. The only way to construct these types is
//! with [`make_guard!`] or `unsafe` code.
//!
//! ```rust
//! use generativity::{Id, make_guard};
//! struct Struct<'id>(Id<'id>);
//! make_guard!(a);
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

use core_::fmt;
use core_::marker::PhantomData;

#[doc(hidden)]
/// NOT STABLE PUBLIC API. Previously Used by the expansion of [`make_guard!`].
pub extern crate core as core_;

/// A phantomdata-like type taking a single invariant lifetime.
///
/// Used to manipulate and store the unique invariant lifetime obtained from
/// [`Guard`]. Use `guard.into()` to create a new `Id`.
///
/// Holding `Id<'id>` indicates that the lifetime `'id` is a trusted brand.
/// `'id` will not unify with another trusted brand lifetime unless it comes
/// from the same original brand (i.e. the same invocation of [`make_guard!`]).
#[repr(transparent)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Id<'id> {
    phantom: PhantomData<fn(&'id ()) -> &'id ()>,
}

impl<'id> Id<'id> {
    /// Construct an `Id` with an unbounded lifetime.
    ///
    /// You should not need to use this function; use [`make_guard!`] instead.
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
/// In effect, this means that `'id` is a "generative brand". Use [`make_guard`]
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
    /// You should not need to use this function; use [`make_guard!`] instead.
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
/// NOT STABLE PUBLIC API. Used by the expansion of [`make_guard!`].
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
/// NOT STABLE PUBLIC API. Used by the expansion of [`make_guard!`].
impl<'id> LifetimeBrand<'id> {
    #[doc(hidden)]
    #[inline(always)]
    /// NOT STABLE PUBLIC API. Used by the expansion of [`make_guard!`].
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
/// Multiple `make_guard` lifetimes will always fail to unify:
///
/// ```rust,compile_fail,E0597
/// # // trybuild ui test tests/ui/crossed_streams.rs
/// # use generativity::make_guard;
/// make_guard!(a);
/// make_guard!(b);
/// dbg!(a == b); // ERROR (here == is a static check)
/// ```
#[macro_export]
macro_rules! make_guard {
    ($name:ident) => {
        // SAFETY: The lifetime given to `$name` is unique among trusted brands.
        // We know this because of how we carefully control drop timing here.
        // The branded lifetime's end is bound to be no later than when the
        // `branded_place` is invalidated at the end of scope, but also must be
        // no sooner than `lifetime_brand` is dropped, also at the end of scope.
        // Some other variant lifetime could be constrained to be equal to the
        // brand lifetime, but no other lifetime branded by `make_guard!` can,
        // as its brand lifetime has a distinct drop time from this one. QED
        let branded_place = unsafe { $crate::Id::new() };
        #[allow(unused)]
        let lifetime_brand = unsafe { $crate::LifetimeBrand::new(&branded_place) };
        let $name = unsafe { $crate::Guard::new(branded_place) };
    };
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
            T: Send + Sync + Unpin + UnwindSafe + RefUnwindSafe,
        {
        }

        make_guard!(a);
        assert_oibits(&a);
        let id: Id<'_> = a.into();
        assert_oibits(&id);

        // const compatible (e.g. const_refs_to_cell, const destructor)
        const fn _const_id(_: Id<'_>) {}
        const fn _const_ref_id(_: &'_ Id<'_>) {}
        const fn _const_guard(_: Guard<'_>) {}
        const fn _const_ref_guard(_: &'_ Guard<'_>) {}
    }
}
