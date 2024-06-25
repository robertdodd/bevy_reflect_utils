# Bevy Reflect Utils

A small, plugin-less utility library making it easier to work with reflection
in [bevy](https://bevyengine.org/).

---

## Development

> [!WARNING]
> UNDER DEVELOPMENT, EXPECT BREAKING CHANGES

## Simple Example

```rust
use bevy::prelude::*;
use bevy_reflect_utils::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<ExampleResource>()
        .add_systems(Startup, setup)
        // IMPORTANT: The types you want to operate on must be registered
        .register_type::<ExampleResource>()
        .run();
}

// IMPORTANT: The types you operate on must derive `Reflect`
#[derive(Resource, Reflect, Debug, Default)]
#[reflect(Resource, Default, Debug)]
pub struct ExampleResource {
    value: bool,
}

fn setup(world: &mut World) {
    // Define a `ReflectTarget` pointing to `ExampleResource::value`
    let target = ReflectTarget::new_resource::<ExampleResource>("value");

    // Read the initial value
    let initial_value = target.read_value::<bool>(world).unwrap();
    println!("initial value: {}", initial_value);

    // Set a new value
    target.set_value(world, !initial_value).unwrap();

    // Read the new value
    let new_value = target.read_value::<bool>(world).unwrap();
    println!("new value: {}", new_value);
}
```

This example will print the following:

```
initial value: false
new value: true
```

You can run this same example with:

```shell
cargo run --example simple
```

## Recommended Usage

Use `Commands` to perform reflection when possible. Use exclusive systems
when you can't avoid it.

For example, to update a value when a button is clicked:

```rust
fn handle_click_events(mut commands: &mut Commands) {
    // if button was clicked...
    let target = ReflectTarget::new_resource::<ExampleResource>("value");
    commands.add(move |world: &mut World| {
        match target.set_value(world, true) {
            Ok(ReflectSetSuccess::Changed) => info!("Success"),
            Ok(ReflectSetSuccess::NoChanges) => warn!("Value not changed"),
            Err(err) => error!("{err:?}"),
        }
    });
}
```

## `ReflectTarget` Target Types

Create a `ReflectTarget` referencing a field on an `Entity` and `Component`:

```rust
let target = ReflectTarget::new_component::<ExampleComponent>(entity, "value");
```

Create a `ReflectTarget` referencing a field on a `Resource`:

```rust
let target = ReflectTarget::new_resource::<ExampleResource>("value");
```

## `ReflectTarget` Operations

`ReflectTarget` provides the following operations:

### Read Value

> Requires knowing the underlying type.

```rust
target.read_value::<f32>(world);
```

Return Value:

```rust
Result<f32, ReflectError>
```

### Set Value

> Requires knowing the underlying type.

```rust
target.set_value(world, 0.5);
```

Return Value:

```rust
Result<ReflectSetSuccess, ReflectError>
```

### Toggle Between Enum Variant

Toggle between the previous/next enum variants.

Also works with data variants, provided the variant implements `Default`.

> Does not require knowing the underlying type.<br />
> **Important:** Does not wrap around when reaching the beginning or end of
> the list of variants.

```rust
target.toggle_enum_variant(world, EnumDirection::Forward);
target.toggle_enum_variant(world, EnumDirection::Backward);
```

Return Value:

```rust
Result<ReflectSetSuccess, ReflectError>
```

### Read Enum Variant Name

> Does not require knowing the underlying type.

```rust
target.read_enum_variant_name(world);
```

Return Value:

```rust
Result<String, ReflectError>
```

### Read Serialized Value

> Does not require knowing the underlying type.

```rust
target.read_value_serialized(world);
```

Return Value:

```rust
Result<String, ReflectError>
```

Example:

```rust
Ok("{\"f32\":0.5}")
```

### Set Serialized Value

> Does not require knowing the underlying type.

```rust
target.set_value_serialized(world, "{\"f32\":0.5}".to_string());
```

Return Value:

```rust
Result<ReflectSetSuccess, ReflectError>
```

### Partial Equality Against a Serialized Value

> Does not require knowing the underlying type.

```rust
target.partial_eq_serialized(world, "{\"f32\":0.5}".to_string());
```

Return Value:

```rust
Result<bool, ReflectError>
```

## Errors

The primary error type is [`ReflectError`](https://github.com/robertdodd/bevy_reflect_utils/blob/master/src/errors.rs).

## Set Value Return Type

Most operations that set a value have the followign return type:

```rust
Result<ReflectSetSuccess, ReflectError>
```

Where [`ReflectSetSuccess`](https://github.com/robertdodd/bevy_reflect_utils/blob/master/src/errors.rs)
allows you know whether the field was changed by the operation:

```rust
pub enum ReflectSetSuccess {
    Changed,
    NoChanges,
}
```

## Compatible Bevy versions

| `bevy_reflect_utils` | `bevy` |
|:---------------------|:-------|
| `0.x`                | `0.13` |

## License

Dual-licensed under either of

- Apache License, Version 2.0,
  ([LICENSE-APACHE](https://github.com/robertdodd/bevy_round_ui/blob/master/LICENSE-APACHE) or
  https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](https://github.com/robertdodd/bevy_round_ui/blob/master/LICENSE-MIT) or
  https://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
