#![cfg_attr(not(test), no_std)]

use core::{fmt, marker::PhantomData};

/// A phantomdata-like type taking a single invariant lifetime.
///
/// Used to manipulate and store the unique invariant lifetime produce by `Guard`.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Id<'id> {
    phantom: PhantomData<&'id mut &'id fn(&'id ()) -> &'id ()>,
}

impl<'id> Id<'id> {
    /// Do not use this function; use the `guard!` macro instead.
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
    fn from(_guard: Guard<'id>) -> Self {
        Id {
            phantom: PhantomData,
        }
    }
}

/// An invariant lifetime phantomdata that is guaranteed to be unique.
///
/// In effect, this means that `'id` is a "generative brand"
#[derive(Eq, PartialEq)]
pub struct Guard<'id> {
    #[allow(unused)]
    id: Id<'id>,
}

impl<'id> Guard<'id> {
    /// Do not use this function; use the `guard!` macro instead.
    pub unsafe fn new(id: Id<'id>) -> Guard<'id> {
        Guard { id }
    }
}

impl<'id> fmt::Debug for Guard<'id> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("#[unique] 'id").finish()
    }
}

/// Create a `Guard` with a unique lifetime.
///
/// Multiple invocations will not unify:
///
/// ```rust,compile_fail
/// # use generativity::make_guard;
/// make_guard!(a);
/// make_guard!(b);
/// dbg!(a == b); // ERROR (here == is a static check)
/// ```
#[macro_export]
macro_rules! make_guard {
    ($name:ident) => {
        let tag = unsafe { $crate::Id::new() };
        let _guard;
        let $name = unsafe { $crate::Guard::new(tag) };
        {
            if false {
                #[allow(non_camel_case_types)]
                struct make_guard<'id>(&'id $crate::Id<'id>);
                impl<'id> ::core::ops::Drop for make_guard<'id> {
                    fn drop(&mut self) {}
                }
                _guard = make_guard(&tag);
            }
        }
    };
}

#[test]
#[allow(clippy::eq_op)]
fn dont_error_in_general() {
    make_guard!(a);
    make_guard!(b);
    dbg!(a == a);
    dbg!(b == b); // OK
}
