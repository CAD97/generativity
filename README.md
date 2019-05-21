# Generativity

[![Chat on Discord](https://img.shields.io/badge/-chat-26262b.svg?style=popout&logo=discord)][Discord]
[![Travis status](https://img.shields.io/travis/com/CAD97/generativity.svg?style=popout&logo=travis)][Travis]
[![Coverage on CodeCov](https://img.shields.io/badge/-coverage-fo1f7a.svg?style=popout&logo=Codecov)][Codecov]

Generativity refers to the creation of a unique lifetime: one that the Rust
borrow checker will not unify with any other lifetime. This can be used to
brand types such that you know that you, and not another copy of you, created
them. This is required for unchecked indexing and similar tricks.

The first step of achieving generativity is an [invariant][variance] lifetime.
Then the consumer must not be able to unify that lifetime with any other lifetime.

Originally, this was achieved with a closure. If you had the user write their code in a
`for<'a> fn(Invariant<'a>) -> _` callback, the local typechecking within this callback
is forced to assume that it can be handed _any_ lifetime (the `for<'a>` bound), and that
it cannot possibly use another lifetime, even `'static`, in its place (the invariance).

This crate implements a different approach using macros and a `Drop`-based scope guard.
When you call `generativity::make_guard!` to make a unique lifetime guard, the macro
first creates an `Id` holding the invariant lifetime. It then puts within a `Drop` type
a `&'id Id<'id>` reference. This ties the lifetime to a concrete lifetime: the lifetime
from the creation of the tag to its `drop` at the end of the containing scope. (As the
tag is encapsulated in the macro hygiene, it cannot be dropped sooner.) A different
invocation of the macro has a different start and end timing to its tag lifetime, so the
lifetimes cannot be unified. These types have no safe way to construct them other than
via `make_guard!`, thus the lifetime is guaranteed unique.

This effectively does the same thing as wrapping the rest of the function in an
immediately invoked closure. We do some pre-work (create the tag and give the caller the
guard), run the user code (the code after here, the closure previously), and then run
some cleanup code after (in the drop implementation). This same technique of a macro
hygiene hidden `impl Drop` can be used by any API that would normally use a closure
argument, such as crossbeam's scoped threads, to make the containing scope the safe,
wrapped scope if they so desire the trade-off of macro versus closure indentation.

## License

Licensed under either of

- Apache License, Version 2.0, (<LICENSE-APACHE> or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license (<LICENSE-MIT> or <http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

## Coverage:

[![Sunburst Coverage](https://codecov.io/gh/CAD97/generativity/graphs/sunburst.svg)][Codecov]

  [Discord]: <https://discord.gg/FuPE9JE>
  [Travis]: <https://travis-ci.com/CAD97/generativity>
  [Codecov]: <https://codecov.io/gh/CAD97/generativity>
  
  [variance]: <https://doc.rust-lang.org/nomicon/subtyping.html#variance>
