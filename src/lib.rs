#![deny(missing_docs, warnings)]

//! Lazily-Evaluated, Order-Independent Plugins for Extensible Types.

extern crate typemap;

use std::any::Any;
use typemap::{TypeMap, Key};

/// Implementers of this trait can act as plugins for other types, via `OtherType::get<P>()`.
///
/// To create a plugin, implement this trait and provide an empty implementation
/// of `Key` to associate the plugin with its return type, `Key::Value`.
pub trait Plugin<E: ?Sized>: Key {
    /// The error type associated with this plugin.
    type Error;

    /// Create the plugin from an instance of the extended type.
    ///
    /// While `eval` is given a mutable reference to the extended
    /// type, it is important for implementers to remember that
    /// the result of `eval` is usually cached, so care should
    /// be taken when doing mutation on the extended type.
    fn eval(&mut E) -> Result<Self::Value, Self::Error>;
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
    /// If plugin creation fails, an error is returned.
    ///
    /// `P` is the plugin type.
    fn get<P: Plugin<Self>>(&mut self) -> Result<P::Value, P::Error>
    where P::Value: Clone + Any, Self: Extensible {
        self.get_ref::<P>().map(|v| v.clone())
    }

    /// Return a reference to the plugin's produced value.
    ///
    /// The plugin will be created if it doesn't exist already.
    /// If plugin creation fails an error is returned.
    ///
    /// `P` is the plugin type.
    fn get_ref<P: Plugin<Self>>(&mut self) -> Result<&P::Value, P::Error>
    where P::Value: Any, Self: Extensible {
        self.get_mut::<P>().map(|mutref| &*mutref)
    }

    /// Return a mutable reference to the plugin's produced value.
    ///
    /// The plugin will be created if it doesn't exist already.
    /// If plugin creation fail an error is returned.
    ///
    /// `P` is the plugin type.
    fn get_mut<P: Plugin<Self>>(&mut self) -> Result<&mut P::Value, P::Error>
    where P::Value: Any, Self: Extensible {
        use typemap::Entry::{Occupied, Vacant};

        if self.extensions().contains::<P>() {
            return Ok(self.extensions_mut().get_mut::<P>().unwrap());
        }

        P::eval(self).map(move |data| {
            match self.extensions_mut().entry::<P>() {
                Vacant(entry) => entry.insert(data),
                Occupied(..) => panic!("Unreachable.")
            }
        })
    }

    /// Create and evaluate a once-off instance of a plugin.
    fn compute<P: Plugin<Self>>(&mut self) -> Result<P::Value, P::Error> {
        <P as Plugin<Self>>::eval(self)
    }
}

#[cfg(test)]
mod test {
    extern crate void;

    use test::void::{Void, ResultVoidExt};

    use typemap::{TypeMap, Key};
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
            #[derive(PartialEq, Debug, Clone)]
            struct $v(i32);

            impl Key for $t { type Value = $t; }

            impl Plugin<Extended> for $t {
                type Error = Void;

                fn eval(_: &mut Extended) -> Result<$t, Void> {
                    Ok($v($v2))
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
        assert_eq!(extended.get::<One>(),   Ok(One(1)));
        assert_eq!(extended.get::<Two>(),   Ok(Two(2)));
        assert_eq!(extended.get::<Three>(), Ok(Three(3)));
    }

    #[test] fn test_resize() {
        let mut extended = Extended::new();
        extended.get::<One>().void_unwrap();
        extended.get::<Two>().void_unwrap();
        extended.get::<Three>().void_unwrap();
        extended.get::<Four>().void_unwrap();
        extended.get::<Five>().void_unwrap();
        extended.get::<Six>().void_unwrap();
        extended.get::<Seven>().void_unwrap();
        extended.get::<Eight>().void_unwrap();
        extended.get::<Nine>().void_unwrap();
        extended.get::<Ten>().void_unwrap();
        assert_eq!(extended.get_ref::<One>(), Ok(&One(1)))
    }

    #[test] fn test_custom_return_type() {
        let mut extended = Extended::new();

        // Define a struct.
        struct IntPlugin;

        // Map it onto an `i32` value.
        impl Key for IntPlugin { type Value = i32; }

        // Define the plugin evaluation function.
        impl Plugin<Extended> for IntPlugin {
            type Error = Void;

            fn eval(_: &mut Extended) -> Result<i32, Void> {
                Ok(0i32)
            }
        }
        assert_eq!(extended.get::<IntPlugin>().void_unwrap(), 0i32);
    }
}

