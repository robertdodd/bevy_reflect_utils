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
