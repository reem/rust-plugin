#![license = "MIT"]
#![deny(missing_doc)]
#![deny(warnings)]

#![feature(macro_rules)]

//! Lazily-Evaluated, Order-Independent Plugins for Extensible Types.

use std::any::{Any, AnyMutRefExt, AnyRefExt};
use std::intrinsics::TypeId;
use std::collections::HashMap;

macro_rules! try_option (
    ($e:expr) => {
        match $e {
            Some(v) => v,
            None => return None
        }
    }
)

/// Defines an interface that extensible types must implement.
///
/// Extensible types must contain a TypeMap.
pub trait Extensible {
    /// Get a reference to the type's extension storage.
    fn extensions(&self) -> &TypeMap;

    /// Get a mutable reference to the type's extension storage.
    fn extensions_mut(&mut self) -> &mut TypeMap;

    /// Creates, stores and returns an instance of T if construction of T
    /// through T's implementation of create succeeds, otherwise None.
    fn get<T: PluginFor<Self> + 'static>(&mut self) -> Option<&T> {
        let found = self.extensions().contains::<T>();
        if found {
            return self.extensions().find();
        }
        let t = try_option!(PluginFor::create(self));
        self.extensions_mut().insert::<T>(t);
        self.get()
    }

    /// Creates, stores and returns a mutable ref T if construction of T
    /// through T's implementation of create succeeds, otherwise None.
    fn get_mut<T: PluginFor<Self> + 'static>(&mut self) -> Option<&mut T> {
        let found = self.extensions().contains::<T>();
        if found {
            return self.extensions_mut().find_mut();
        }
        let t = try_option!(PluginFor::create(self));
        self.extensions_mut().insert::<T>(t);
        self.get_mut()
    }
}

/// Implementations of this trait can act as plugins for `T`, via `T::get<P>()`
pub trait PluginFor<T: Extensible> {
    /// Create Self from an instance of T. This will be called only once.
    fn create(&T) -> Option<Self>;
}

/// A map of which can contain zero or one instances of any type.
pub struct TypeMap {
    map: HashMap<TypeId, Box<Any>>
}

impl TypeMap {
    /// Create a new TypeMap
    pub fn new() -> TypeMap { TypeMap { map: HashMap::new() } }

    /// Find and get a reference to an instance of T if it exists.
    pub fn find<T: 'static>(&self) -> Option<&T> { self.map.find(&TypeId::of::<T>()).and_then(|any| any.downcast_ref()) }

    /// Find and get a mutable reference to an instance of T if it exists.
    pub fn find_mut<T: 'static>(&mut self) -> Option<&mut T> { self.map.find_mut(&TypeId::of::<T>()).and_then(|any| any.downcast_mut()) }

    /// Insert an instance of T.
    pub fn insert<T: 'static>(&mut self, t: T) { self.map.insert(TypeId::of::<T>(), box t as Box<Any>); }

    /// Does the map contain an instance of T?
    pub fn contains<T: 'static>(&self) -> bool { self.map.contains_key(&TypeId::of::<T>()) }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use super::{TypeMap, Extensible, PluginFor};

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
        fn extensions_mut(&mut self) -> &mut TypeMap { &mut self.map }
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
        let mut extended = Extended::new();
        assert_eq!(extended.get::<One>(),   Some(&One(1)))
        assert_eq!(extended.get::<Two>(),   Some(&Two(2)))
        assert_eq!(extended.get::<Three>(), Some(&Three(3)))
    }

    #[test] fn test_resize() {
        let mut extended = Extended::new();
        extended.get::<One>();
        extended.get::<Two>();
        extended.get::<Three>();
        extended.get::<Four>();
        extended.get::<Five>();
        extended.get::<Six>();
        extended.get::<Seven>();
        extended.get::<Eight>();
        extended.get::<Nine>();
        extended.get::<Ten>();
        assert_eq!(extended.get::<One>(), Some(&One(1)))
    }
}

