# Generativity [![Chat on Discord](https://img.shields.io/discord/496481045684944897?style=flat&logo=discord)][Discord]

Generativity refers to the creation of a unique lifetime: one that the Rust
borrow checker will not unify with any other lifetime. This can be used to
brand types such that you know that you, and not another copy of you, created
them. This is required for sound unchecked indexing and similar tricks.

The first step of achieving generativity is an [invariant][variance] lifetime.
Then the consumer must not be able to unify that lifetime with any other lifetime.

Traditionally, this is achieved with a closure. If you have the user write their code in a
`for<'a> fn(Invariant<'a>) -> _` callback, the local typechecking within this callback
is forced to assume that it can be handed _any_ lifetime (the `for<'a>` bound), and that
it cannot possibly use another lifetime, even `'static`, in its place (the invariance).

This crate implements a different approach using macros and a `Drop`-based scope guard.
When you call `generativity::make_guard!` to make a unique lifetime guard, the macro
first creates an `Id` holding an invariant lifetime. It then puts within a `Drop` type
a `&'id Id<'id>` reference. A different invocation of the macro has a different end timing
to its tag lifetime, so the lifetimes cannot be unified. These types have no safe way to
construct them other than via `make_guard!`, thus the lifetime is guaranteed unique.

This effectively does the same thing as wrapping the rest of the function in an
immediately invoked closure. We do some pre-work (create the tag and give the caller the
guard), run the user code (the code after here, the closure previously), and then run
some cleanup code after (in the drop implementation). This same technique of a macro
hygiene hidden `impl Drop` can be used by most APIs that would normally use a closure
argument, enabling them to avoid introducing a closure boundary to control flow "effect"s
such as `async.await` and `?`. But this also comes with a subtle downside: being able to
`.await` means that locals could be forgotten instead of dropped, and thus the drop glue
must not be relied on to run for soundness. This is okay for this crate, since the actual
drop impl is a no-op that just exists for the lifetime analysis impacts, but it means that
more interesting cases like scoped threads still need to use a closure callback to ensure
their soundness-critical cleanup gets run (e.g. to wait on and join the scoped threads).

It's important to note that lifetimes are only trusted to carry lifetime brands when they
are in fact invariant. Variant lifetimes, such as `&'a T`, can still be shrunk to fit the
branded lifetime; `&'static T` can be used where `&'a T` is expected, for *any* `'a`.

## How does it work?

```rust
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
// implicit when exiting scope: drop(lifetime_brand), as LifetimeBrand: Drop
let $name = unsafe { $crate::Guard::new(branded_place) };
```

A previous version of this crate emitted more code in the macro, and defined the
`LifetimeBrand` type inline. This improved compiler errors on compiler versions
from the time, but the current compiler produces an equivalently good or better
error with `LifetimeBrand` defined in the `generativity` crate.

The `LifetimeBrand` type is *not public API* and must not be used directly. It
is not covered by stability guarantees; only usage of `make_guard!` and other
documented APIs are considered stable.

## Huge generativity disclaimer

That last point above is ***VERY*** important. We _cannot_ guarantee that the
lifetime is fully unique. We can only guarantee that it's unique among trusted
carriers. This applies equally to the traditional method:

```rust
fn scope<F>(f: F)
where F: for<'id> FnOnce(Guard<'id>)
{
    make_guard!(guard);
    f(guard);
}

fn unify<'a>(_: &'a (), _: &Guard<'a>) {
    // here, you have two `'a` which are equivalent to `guard`'s `'id`
}

fn main() {
    let place = ();
    make_guard!(guard);
    unify(&place, &guard);
    scope(|guard| {
        unify(&place, &guard);
    })
}
```

Other variant lifetimes can still shrink to unify with `'id`. What all this
means is that you can't just trust any old `'id` lifetime floating around. It
has to be carried in a trusted carrier, and one that wasn't created from an
untrusted lifetime.

## Impl Disclaimer

This relies on dead code (an `#[inline(always)]` no-op drop) to impact borrow
checking. In theory, a sufficiently advanced borrow checker looking at the CFG
after some inlining would be able to see that dropping `lifetime_brand` doesn't
require the captured lifetime to be live, and destroy the uniqueness guarantee
which we've created. This would be much more difficult with the higher-ranked
closure formulation, but would still theoretically be possible with sufficient
inlining. Thankfully, based on the direction around the unstable "borrowck
eyepatch" which is the reason e.g. `Box<&'a T>` can be dropped at end of scope
despite the `&'a T` borrow being invalidated beforehand, and the further
stability implications of inferring whether a generic is "used" by `Drop::drop`,
it seems like any such weakening of an explicit `impl Drop` "using" captured
lifetimes in the eyes of borrowck will be opt-in. This crate won't opt in to
such a feature, and thus will remain sound.

## Minimum supported Rust version

The crate currently requires Rust 1.56. I have no intent of increasing the
compiler version requirement of this crate beyond this. However, this is only
guaranteed within a given minor version number.

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

  [Discord]: <https://discord.gg/FuPE9JE>
  [variance]: <https://doc.rust-lang.org/nomicon/subtyping.html#variance>
