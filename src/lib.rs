#![allow(unstable)]
#![deny(missing_docs, warnings)]

//! Lazily-Evaluated, Order-Independent Plugins for Extensible Types.

extern crate typemap;
extern crate phantom;
use typemap::{TypeMap, Key};

pub use phantom::Phantom;

/// Implementers of this trait can act as plugins for other types, via `OtherType::get<P>()`.
///
/// To create a plugin, implement this trait and provide an empty implementation
/// of `Key` to associate the plugin with its return type, `Key::Value`.
pub trait Plugin<E: ?Sized>: Key {
    /// Create the plugin from an instance of the extended type.
    ///
    /// While `eval` is given a mutable reference to the extended
    /// type, it is important for implementers to remember that
    /// the result of `eval` is usually cached, so care should
    /// be taken when doing mutation on the extended type.
    fn eval(&mut E, Phantom<Self>) -> Option<Self::Value>;
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
pub trait Pluggable {
    /// Return a copy of the plugin's produced value.
    ///
    /// The plugin will be created if it doesn't exist already.
    /// If plugin creation fails, `None` is returned.
    ///
    /// `P` is the plugin type.
    fn get<P: Plugin<Self>>(&mut self) -> Option<P::Value>
    where P::Value: Clone + 'static, Self: Extensible {
        self.get_ref::<P>().cloned()
    }

    /// Return a reference to the plugin's produced value.
    ///
    /// The plugin will be created if it doesn't exist already.
    /// If plugin creation fails, `None` is returned.
    ///
    /// `P` is the plugin type.
    fn get_ref<P: Plugin<Self>>(&mut self) -> Option<&P::Value>
    where P::Value: 'static, Self: Extensible {
        self.get_mut::<P>().map(|mutref| &*mutref)
    }

    /// Return a mutable reference to the plugin's produced value.
    ///
    /// The plugin will be created if it doesn't exist already.
    /// If plugin creation fails, `None` is returned.
    ///
    /// `P` is the plugin type.
    fn get_mut<P: Plugin<Self>>(&mut self) -> Option<&mut P::Value>
    where P::Value: 'static, Self: Extensible {
        use typemap::Entry::{Occupied, Vacant};
        use std::intrinsics::unreachable;

        if self.extensions().contains::<P>() {
            return self.extensions_mut().get_mut::<P>();
        }

        Plugin::eval(self, Phantom::<P>).map(move |data| {
            match self.extensions_mut().entry::<P>() {
                Vacant(entry) => entry.insert(data),
                Occupied(..) => unsafe { unreachable() }
            }
        })
    }

    /// Create and evaluate a once-off instance of a plugin.
    fn compute<P: Plugin<Self>>(&mut self) -> Option<P::Value> {
        Plugin::eval(self, Phantom::<P>)
    }
}

#[cfg(test)]
mod test {
    use typemap::{TypeMap, Key};
    use phantom::Phantom;
    use super::{Extensible, Plugin, Pluggable};

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

    impl Pluggable for Extended {}

    macro_rules! generate_simple_plugin (
        ($t:ty, $v:ident, $v2:expr) => {
            #[derive(PartialEq, Show, Clone)]
            struct $v(i32);

            impl Key for $t { type Value = $t; }

            impl Plugin<Extended> for $t {
                fn eval(_: &mut Extended, _: Phantom<$t>) -> Option<$t> {
                    Some($v($v2))
                }
            }
        }
    );

    generate_simple_plugin!(One, One, 1);
    generate_simple_plugin!(Two, Two, 2);
    generate_simple_plugin!(Three, Three, 3);
    generate_simple_plugin!(Four, Four, 4);
    generate_simple_plugin!(Five, Five, 5);
    generate_simple_plugin!(Six, Six, 6);
    generate_simple_plugin!(Seven, Seven, 7);
    generate_simple_plugin!(Eight, Eight, 8);
    generate_simple_plugin!(Nine, Nine, 9);
    generate_simple_plugin!(Ten, Ten, 10);

    #[test] fn test_simple() {
        let mut extended = Extended::new();
        assert_eq!(extended.get::<One>(),   Some(One(1)));
        assert_eq!(extended.get::<Two>(),   Some(Two(2)));
        assert_eq!(extended.get::<Three>(), Some(Three(3)));
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
        impl Key for IntPlugin { type Value = i32; }

        // Define the plugin evaluation function.
        impl Plugin<Extended> for IntPlugin {
            fn eval(_: &mut Extended, _: Phantom<IntPlugin>) -> Option<i32> {
                Some(0i32)
            }
        }
        assert_eq!(extended.get::<IntPlugin>().unwrap(), 0i32);
    }
}

