use generativity::{guard, Guard};

fn unify<'id>(_: Guard<'id>, _: Guard<'id>) {
    unreachable!()
}

fn main() {
    match (guard!(), guard!()) {
        (a, b) => unify(a, b),
    }
}
