#![license = "MIT"]
#![deny(warnings)]

#![feature(macro_rules)]

//! Lazily-Evaluated, Order-Independent Plugins for Extensible Types.

use std::any::{Any, AnyMutRefExt, AnyRefExt};
use std::intrinsics::TypeId;
use std::collections::HashMap;

pub struct TypeMap {
    map: HashMap<TypeId, Box<Any>>
}

impl TypeMap {
    pub fn new() -> TypeMap { TypeMap { map: HashMap::new() } }

    pub fn find<T: 'static>(&self) -> Option<&T> { self.map.find(&TypeId::of::<T>()).and_then(|any| any.downcast_ref()) }

    pub fn find_mut<T: 'static>(&mut self) -> Option<&mut T> { self.map.find_mut(&TypeId::of::<T>()).and_then(|any| any.downcast_mut()) }

    pub fn insert<T: 'static>(&mut self, t: T) { self.map.insert(TypeId::of::<T>(), box t as Box<Any>); }

    pub fn remove<T: 'static>(&mut self) { self.map.remove(&TypeId::of::<T>()); }
}

pub trait Extensible {
    fn extensions(&self) -> &TypeMap;
    fn extensions_mut(&mut self) -> &mut TypeMap;
}

pub trait PluginFor<T: Extensible> {
    fn create(&T) -> Option<Self>;
}

pub trait Get {
    fn get<T: PluginFor<Self> + 'static>(&mut self) -> Option<&T>;
    fn get_mut<T: PluginFor<Self> + 'static>(&mut self) -> Option<&mut T>;
}

macro_rules! try_option (
    ($e:expr) => {
        match $e {
            Some(v) => v,
            None => return None
        }
    }
)

fn compute<E: Extensible, T: PluginFor<E> + 'static>(map: &mut E) -> Option<&T> {
    let t = try_option!(PluginFor::create(map));
    map.extensions_mut().insert::<T>(t);
    map.get()
}

impl<E: Extensible> Get for E {
    fn get<T: PluginFor<E> + 'static>(&mut self) -> Option<&T> {
        {
            let found = self.extensions().find();
            if found.is_some() {
                return found;
            }
        }
        compute(self)
    }

    fn get_mut<T: PluginFor<E> + 'static>(&mut self) -> Option<&mut T> {
        fail!()
    }
}

#[cfg(test)]
mod test {
    use std::cell::UnsafeCell;
    use std::mem;
    use std::collections::HashMap;
    use super::{TypeMap, Extensible, PluginFor, Get};

    struct Extended {
        map: TypeMap
    }

    impl Extended {
        fn new() -> Extended {
            Extended { map: TypeMap { map: HashMap::with_capacity(2) } }
        }
    }

    impl Extensible for Extended {
        fn extensions(&self) -> &TypeMap { &self.map }
        fn extensions_mut(&self) -> &mut TypeMap { &mut self.map }
    }

    macro_rules! generate_plugin (
        ($t:ty, $v:ident, $v2:expr) => {
            #[deriving(PartialEq, Show)]
            struct $v(uint);

            impl PluginFor<Extended> for $t {
                fn create(_: &Extended) -> Option<$t> { Some($v($v2)) }
            }
        }
    )

    generate_plugin!(One, One, 1)
    generate_plugin!(Two, Two, 2)
    generate_plugin!(Three, Three, 3)
    generate_plugin!(Four, Four, 4)
    generate_plugin!(Five, Five, 5)
    generate_plugin!(Six, Six, 6)
    generate_plugin!(Seven, Seven, 7)
    generate_plugin!(Eight, Eight, 8)
    generate_plugin!(Nine, Nine, 9)
    generate_plugin!(Ten, Ten, 10)

    #[test] fn test_simple() {
        let extended = Extended::new();
        assert_eq!(extended.get::<One>(),   Some(&One(1)))
        assert_eq!(extended.get::<Two>(),   Some(&Two(2)))
        assert_eq!(extended.get::<Three>(), Some(&Three(3)))
    }

    #[test] fn test_resize() {
        let extended = Extended::new();
        let one = extended.get::<One>();
        extended.get::<Two>();
        extended.get::<Three>();
        extended.get::<Four>();
        extended.get::<Five>();
        extended.get::<Six>();
        extended.get::<Seven>();
        extended.get::<Eight>();
        extended.get::<Nine>();
        extended.get::<Ten>();
        assert_eq!(*one.unwrap(), One(1));
    }
}

