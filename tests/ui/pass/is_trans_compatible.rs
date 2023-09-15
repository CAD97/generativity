// see cad97/generativity#13
#![deny(repr_transparent_external_private_fields)]

use generativity::Id;

pub struct BOption<'id, T>(Option<T>, Id<'id>);

fn main() {}
