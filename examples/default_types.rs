extern crate plugin;
extern crate typemap;

use plugin::{Extensible, Plugin, Pluggable, Phantom};
use typemap::{TypeMap, Key};

struct Struct {
    map: TypeMap
}

impl Extensible for Struct {
    fn extensions(&self) -> &TypeMap {
        &self.map
    }
    fn extensions_mut(&mut self) -> &mut TypeMap {
        &mut self.map
    }
}

impl Pluggable for Struct {}

#[derive(Clone, Show)]
struct IntPlugin {
    field: i32
}

impl Key for IntPlugin { type Value = IntPlugin; }

impl Plugin<Struct> for IntPlugin {
    fn eval(_: &mut Struct, _: Phantom<IntPlugin>) -> Option<IntPlugin> {
        Some(IntPlugin { field: 7i32 })
    }
}

fn main() {
    let mut x = Struct { map: TypeMap::new() };
    println!("{:?}", x.get_ref::<IntPlugin>());
}

