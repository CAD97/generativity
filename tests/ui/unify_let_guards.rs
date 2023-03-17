use generativity::{guard, Guard};

fn unify<'id>(_: Guard<'id>, _: Guard<'id>) {
    unreachable!()
}

fn main() {
    guard!(let a, b);
    unify(a, b);
}
