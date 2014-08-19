# Plugin

> Type-Safe, Lazily Evaluated, Plugins for Extensible Types

Plugins provide a consistent interface for mixin methods. You can use a plugin anywhere you would use a "mixin" trait and an implementation.

## Example Usage

```rust
// Define a struct.
struct IntPlugin;

// Map it onto an `i32` value.
impl Assoc<i32> for IntPlugin {}

// Define the plugin evaluation function.
// `Extended` is a type that implements `Extensible`.
impl PluginFor<Extended, i32> for IntPlugin {
    fn eval(_: &Extended, _: Phantom<IntPlugin>) -> Option<i32> {
        Some(0i32)
    }
}
assert_eq!(extended.get::<IntPlugin>().unwrap(), 0i32);
```

To do the same thing with a trait, one could do:

```rust
trait IntProducer {
    fn get_int_value(&self) -> Option<i32>;
}

impl IntProducer for Extended {
    fn get_int_value(&self) -> Option<i32> {
        Some(0i32)
    }
}
```

Although using a raw trait is less code, plugins provide the following advantages:

* Automatic caching of values. Calling a method again is a constant time operation! This is particularly useful in pipeline structures where only the extensible object is passed around.
* A consistent interface, which also allows for neater name clash resolution. Two modules that provide `PluginX` can be differentiated using a module prefix.

```
e.get::<mod1::PluginX>();
e.get::<mod2::PluginX>();
```
