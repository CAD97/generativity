// see cad97/generativity#13 and rust-lang/rust#78586
// if this fails in crater *please* ping me; I *want* this to not hit the lint!
#![deny(repr_transparent_external_private_fields)] // this should not trigger

use generativity::{make_guard, Id};

#[repr(transparent)]
pub struct BOption<'id, T>(Option<T>, Id<'id>); // this should work

fn main() {
    make_guard!(a);
    let _ = BOption(Some(0), a.into());
}
