extern crate plugin;
extern crate typemap;

use plugin::{Extensible, PluginFor, GetCached, Phantom};
use typemap::{TypeMap, Assoc};

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

#[deriving(Clone, Show)]
struct Plugin {
    field: i32
}
impl Assoc<Plugin> for Plugin {}

impl PluginFor<Struct> for Plugin {
    fn eval(_: &Struct, _: Phantom<Plugin>) -> Option<Plugin> {
        Some(Plugin { field: 7i32 })
    }
}

fn main() {
    let mut x = Struct { map: TypeMap::new() };
    println!("{}", x.get_ref::<Plugin>());
}
