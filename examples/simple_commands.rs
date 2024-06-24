use bevy::prelude::*;

use bevy_reflect_utils::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        // IMPORTANT: The types you want to operate on must be registered
        .register_type::<ExampleResource>()
        .register_type::<ExampleComponent>()
        .run();
}

// IMPORTANT: The types you operate on must derive `Reflect`
#[derive(Resource, Reflect, Debug, Default)]
#[reflect(Resource, Default, Debug)]
pub struct ExampleResource {
    value: bool,
}

// IMPORTANT: The types you operate on must derive `Reflect`
#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component, Default, Debug)]
pub struct ExampleComponent {
    value: i32,
}

fn setup(mut commands: Commands) {
    // Insert a resource that we will operate on via reflection
    commands.insert_resource(ExampleResource { value: false });

    // Spawn an entity with a `ExampleComponent` component that we will operate on via reflection
    let entity = commands.spawn(ExampleComponent { value: 0 }).id();

    // Add a command to update the value of the resource
    commands.add(move |world: &mut World| {
        // Define a `ReflectTarget` pointing to `ExampleResource::value`
        let target = ReflectTarget::new_resource::<ExampleResource>("value");

        // Read the initial value
        let initial_value = target.read_value::<bool>(world).unwrap();
        // Set a new value
        target.set_value(world, !initial_value).unwrap();
        // Read the new value
        let new_value = target.read_value::<bool>(world).unwrap();

        println!(
            "\"ExampleResource::value\" ==> changed from `{}` to `{}`",
            initial_value, new_value
        );
        println!(
            "\"ExampleResource::value\" ==> {}",
            target.read_value_serialized(world).unwrap()
        );
    });

    // Add a command to update the value of the component
    commands.add(move |world: &mut World| {
        // Define a `ReflectTarget` pointing to `ExampleComponent::value` on the entity.
        let target = ReflectTarget::new_component::<ExampleComponent>(entity, "value");

        // Read the initial value of `ExampleComponent::value`
        let initial_value = target.read_value::<i32>(world).unwrap();
        // Set a new value
        target.set_value(world, initial_value + 100).unwrap();
        // Read the new value
        let new_value = target.read_value::<i32>(world).unwrap();

        println!(
            "\"ExampleComponent::value\" ==> changed from `{}` to `{}`",
            initial_value, new_value
        );
        println!(
            "\"ExampleComponent::value\" ==> {}",
            target.read_value_serialized(world).unwrap()
        );
    });
}
