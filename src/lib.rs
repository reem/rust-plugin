#![license = "MIT"]
#![deny(missing_doc)]
#![deny(warnings)]

#![feature(macro_rules)]
#![feature(default_type_params)]

//! Lazily-Evaluated, Order-Independent Plugins for Extensible Types.

extern crate typemap;
extern crate phantom;
use typemap::{TypeMap, Assoc};

pub use phantom::Phantom;

/// Implementers of this trait can act as plugins for other types, via `OtherType::get<P>()`.
///
/// To create a plugin, implement this trait and provide an empty implementation
/// of `Assoc<R>` to associate the plugin with its return type, `R`.
///
/// `E` is the type to extend.
///
/// `R` is the type of the value produced by the plugin.
pub trait PluginFor<E, R = Self>: Assoc<R> {
    /// Create the plugin from an instance of the extended type.
    fn eval(&E, Phantom<Self>) -> Option<R>;
}

/// Defines an interface that extensible types must implement.
///
/// Extensible types must contain a TypeMap.
pub trait Extensible {
    /// Get a reference to the type's extension storage.
    fn extensions(&self) -> &TypeMap;

    /// Get a mutable reference to the type's extension storage.
    fn extensions_mut(&mut self) -> &mut TypeMap;
}

/// An interface for plugins that cache values between calls.
///
/// `R` is the type of the plugin's return value, which must be cloneable.
pub trait GetCached<R: Clone + 'static>: Extensible {
    /// Return a copy of the plugin's produced value.
    ///
    /// The plugin will be created if it doesn't exist already.
    /// If plugin creation fails, `None` is returned.
    ///
    /// `P` is the plugin type, and `R` is the plugin's return type.
    fn get<'a, P: PluginFor<Self, R> + 'static>(&'a mut self) -> Option<R> {
        let f = |value: &'a mut R| -> R {
            value.clone()
        };
        self.get_common::<P, R>(f)
    }

    /// Return a reference to the plugin's produced value.
    ///
    /// The plugin will be created if it doesn't exist already.
    /// If plugin creation fails, `None` is returned.
    ///
    /// `P` is the plugin type, and `R` is the plugin's return type.
    fn get_ref<'a, P: PluginFor<Self, R> + 'static>(&'a mut self) -> Option<&'a R> {
        let f = |value: &'a mut R| -> &'a R {
            &*value
        };
        self.get_common::<P, &'a R>(f)
    }

    /// Return a mutable reference to the plugin's produced value.
    ///
    /// The plugin will be created if it doesn't exist already.
    /// If plugin creation fails, `None` is returned.
    ///
    /// `P` is the plugin type, and `R` is the plugin's return type.
    fn get_mut<'a, P: PluginFor<Self, R> + 'static>(&'a mut self) -> Option<&'a mut R> {
        let f = |value: &'a mut R| -> &'a mut R {
            value
        };
        self.get_common::<P, &'a mut R>(f)
    }

    /// Convenience function for get methods.
    #[doc(hidden)]
    fn get_common<'a, P: PluginFor<Self, R> + 'static, S>(&'a mut self, f: |&'a mut R| -> S)
    -> Option<S> {
        // If a plugin is already registered, extract and return its value.
        let found = self.extensions().contains::<P, R>();
        if found {
            let result = self.extensions_mut().find_mut::<P, R>().unwrap();
            return Some(f(result));
        }
        // Otherwise, register a new plug-in and recurse.
        match PluginFor::eval(self, Phantom::<P>) {
            Some(value) => {
                self.extensions_mut().insert::<P, R>(value);
                self.get_common::<P, S>(f)
            },
            None => None
        }
    }
}

/// An interface for using plugins with non-extensible types.
pub trait Get<R> {
    /// Create and evaluate a once-off instance of a plugin.
    fn compute<P: PluginFor<Self, R> + Assoc<R>>(&self) -> Option<R> {
        PluginFor::eval(self, Phantom::<P>)
    }
}

/// If a plugin is registered for a type, allow it to be used without caching.
impl<T, R> Get<R> for T {}

/// If a plugin is implemented for an extensible type, then you can use all the caching get methods.
impl<E: Extensible, R: Clone + Send> GetCached<R> for E {}

#[cfg(test)]
mod test {
    use typemap::{TypeMap, Assoc};
    use phantom::Phantom;
    use super::{Extensible, PluginFor, GetCached};

    struct Extended {
        map: TypeMap
    }

    impl Extended {
        fn new() -> Extended {
            Extended { map: TypeMap::new() }
        }
    }

    impl Extensible for Extended {
        fn extensions(&self) -> &TypeMap { &self.map }
        fn extensions_mut(&mut self) -> &mut TypeMap { &mut self.map }
    }

    macro_rules! generate_simple_plugin (
        ($t:ty, $v:ident, $v2:expr) => {
            #[deriving(PartialEq, Show, Clone)]
            struct $v(uint);

            impl Assoc<$t> for $t {}

            impl PluginFor<Extended> for $t {
                fn eval(_: &Extended, _: Phantom<$t>) -> Option<$t> {
                    Some($v($v2))
                }
            }
        }
    )

    generate_simple_plugin!(One, One, 1)
    generate_simple_plugin!(Two, Two, 2)
    generate_simple_plugin!(Three, Three, 3)
    generate_simple_plugin!(Four, Four, 4)
    generate_simple_plugin!(Five, Five, 5)
    generate_simple_plugin!(Six, Six, 6)
    generate_simple_plugin!(Seven, Seven, 7)
    generate_simple_plugin!(Eight, Eight, 8)
    generate_simple_plugin!(Nine, Nine, 9)
    generate_simple_plugin!(Ten, Ten, 10)

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

    #[test] fn test_custom_return_type() {
        let mut extended = Extended::new();

        // Define a struct.
        struct IntPlugin;

        // Map it onto an `i32` value.
        impl Assoc<i32> for IntPlugin {}

        // Define the plugin evaluation function.
        impl PluginFor<Extended, i32> for IntPlugin {
            fn eval(_: &Extended, _: Phantom<IntPlugin>) -> Option<i32> {
                Some(0i32)
            }
        }
        assert_eq!(extended.get::<IntPlugin>().unwrap(), 0i32);
    }
}

