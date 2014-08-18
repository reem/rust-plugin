#![license = "MIT"]
#![deny(missing_doc)]
#![deny(warnings)]

#![feature(macro_rules)]

//! Lazily-Evaluated, Order-Independent Plugins for Extensible Types.

extern crate anymap;
use anymap::AnyMap;

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
/// Extensible types must contain a AnyMap.
pub trait Extensible {
    /// Get a reference to the type's extension storage.
    fn extensions(&self) -> &AnyMap;

    /// Get a mutable reference to the type's extension storage.
    fn extensions_mut(&mut self) -> &mut AnyMap;
}

/// Expose an interface for cacheing plugins.
pub trait GetCached<'a>: Extensible {
    /// Creates, stores and returns reference of T if construction of T
    /// through T's implementation of create succeeds, otherwise None.
    fn get_ref<'a, T: PluginFor<Self> + 'static>(&'a mut self) -> Option<&'a T> {
        let f = |x: &'a mut T| -> &'a T {
            &*x
        };
        self.get_common::<T, &'a T>(f)
    }

    /// Creates, stores and returns a mutable ref of T if construction of T
    /// through T's implementation of create succeeds, otherwise None.
    fn get_mut<'a, T: PluginFor<Self> + 'static>(&'a mut self) -> Option<&'a mut T> {
        let f = |x: &'a mut T| -> &'a mut T {
            x
        };
        self.get_common::<T, &'a mut T>(f)
    }

    /// Creates, stores and returns an instance of T if construction of T
    /// through T's implementation of create succeeds, otherwise None.
    fn get<'a, T: PluginFor<Self> + 'static + Clone>(&'a mut self) -> Option<T> {
        let f = |x: &'a mut T| -> T {
            x.clone()
        };
        self.get_common::<T, T>(f)
    }

    #[doc(hidden)]
    fn get_common<'a, T: PluginFor<Self> + 'static, R>(&'a mut self, f: |&'a mut T| -> R) -> Option<R> {
        let found = self.extensions().contains::<T>();
        if found {
            return self.extensions_mut().find_mut::<T>().map(f);
        }
        let plugin = try_option!(PluginFor::create(self));
        self.extensions_mut().insert::<T>(plugin);
        self.get_common::<T, R>(f)
    }
}

/// An interface for getting plugins on non-extensible types.
pub trait Get {
    /// Call the appropriate PluginFor implementation to create an instance
    /// of T.
    fn compute<T: PluginFor<Self>>(&self) -> Option<T> {
        PluginFor::create(self)
    }
}

impl<T> Get for T {}
impl<'a, T: Extensible> GetCached<'a> for T {}

/// Implementations of this trait can act as plugins for `T`, via `T::get<P>()`
pub trait PluginFor<T> {
    /// Create Self from an instance of T. This will be called only once.
    fn create(&T) -> Option<Self>;
}

#[cfg(test)]
mod test {
    use anymap::AnyMap;
    use super::{Extensible, PluginFor, GetCached};

    struct Extended {
        map: AnyMap
    }

    impl Extended {
        fn new() -> Extended {
            Extended { map: AnyMap::new() }
        }
    }

    impl Extensible for Extended {
        fn extensions(&self) -> &AnyMap { &self.map }
        fn extensions_mut(&mut self) -> &mut AnyMap { &mut self.map }
    }

    macro_rules! generate_plugin (
        ($t:ty, $v:ident, $v2:expr) => {
            #[deriving(PartialEq, Show, Clone)]
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
        assert_eq!(extended.get_ref::<One>(),   Some(&One(1)))
        assert_eq!(extended.get_ref::<Two>(),   Some(&Two(2)))
        assert_eq!(extended.get_ref::<Three>(), Some(&Three(3)))
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
        assert_eq!(extended.get_ref::<One>(), Some(&One(1)))
    }
}

