use bevy::prelude::*;

use bevy_reflect_utils::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<ExampleResource>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                handle_input,
                update_label.run_if(resource_exists_and_changed::<ExampleResource>),
            ),
        )
        .register_type::<ExampleResource>()
        .run();
}

#[derive(Resource, Reflect, Debug, Default)]
#[reflect(Resource, Default, Debug)]
pub struct ExampleResource {
    value: i32,
}

#[derive(Component)]
struct ExampleLabel;

/// System that spawns the UI for this example.
fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Spawn a full screen, centered node containing text content
    commands
        .spawn(Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
        })
        .with_children(|p| {
            // Instructions
            p.spawn(Text::new(
                "Use ARROW_LEFT and ARROW_RIGHT to increase or decrease the value, to maximum of 10 and minimum of -10."
            ));

            // Label showing the value of `ExampleResource::value`
            p.spawn(Text::new("Current Value: ")).with_children(|p| {
                p.spawn((ExampleLabel, TextSpan::default()));
            });
        });
}

/// System that listens for input and toggles the value of the enum variant.
fn handle_input(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>) {
    let amount: Option<i32> = if keys.just_pressed(KeyCode::ArrowLeft) {
        Some(-1)
    } else if keys.just_pressed(KeyCode::ArrowRight) {
        Some(1)
    } else {
        None
    };

    if let Some(amount) = amount {
        // Define the reflection target
        let target = ReflectTarget::new_resource::<ExampleResource>("value");

        // We need world access to perform reflection, so add a one-off command to perform the operation.
        commands.queue(move |world: &mut World| {
            // Read the current value via reflection
            let current_value = target.read_value::<i32>(world);

            // Update the value
            let result = current_value.and_then(|current_value| {
                let new_value = (current_value + amount).clamp(-10, 10);
                target.set_value(world, new_value)
            });

            // Log the results of the operation
            match result {
                Ok(ReflectSetSuccess::Changed) => info!("Success"),
                Ok(ReflectSetSuccess::NoChanges) => warn!("Value not changed"),
                Err(err) => error!("{err:?}"),
            }
        });
    }
}

/// System that updates value of the text node to display the value of [`ExampleResource::value`].
fn update_label(
    mut query: Query<&mut TextSpan, With<ExampleLabel>>,
    example: Res<ExampleResource>,
) {
    for mut text in query.iter_mut() {
        text.0 = format!("{:?}", example.value);
    }
}
