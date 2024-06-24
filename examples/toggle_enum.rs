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
        .register_type::<ExampleEnum>()
        .register_type::<ExampleResource>()
        .run();
}

#[derive(Default, Reflect, Debug)]
#[reflect(Default, Debug)]
pub enum ExampleEnum {
    #[default]
    OptionA,
    OptionB,
    Number(i32),
}

#[derive(Resource, Reflect, Debug, Default)]
#[reflect(Resource, Default, Debug)]
pub struct ExampleResource {
    value: ExampleEnum,
}

#[derive(Component)]
struct ExampleLabel;

/// System that spawns the UI for this example.
fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // Spawn a full screen, centered node containing text content
    commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|p| {
            // Instructions
            p.spawn(TextBundle::from_section(
                "Use ARROW_LEFT and ARROW_RIGHT to toggle between enum variants",
                TextStyle::default(),
            ));

            // Label showing the value of `ExampleResource::value`
            p.spawn((
                ExampleLabel,
                TextBundle::from_sections([
                    TextSection::new("Current Value: ", TextStyle::default()),
                    TextSection::new("", TextStyle::default()),
                ]),
            ));
        });
}

/// System that listens for input and toggles the value of the enum variant.
fn handle_input(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>) {
    let direction = if keys.just_pressed(KeyCode::ArrowLeft) {
        Some(EnumDirection::Backward)
    } else if keys.just_pressed(KeyCode::ArrowRight) {
        Some(EnumDirection::Forward)
    } else {
        None
    };

    if let Some(direction) = direction {
        // Define the reflection target
        let target = ReflectTarget::new_resource::<ExampleResource>("value");

        // Toggle the enum variant pointed at by the reflect target.
        // We need world access to perform reflection, so add a one-off command to perform the operation.
        commands.add(move |world: &mut World| {
            let result = target.toggle_reflect_enum(world, direction);
            match result {
                Ok(ReflectSetSuccess::Changed) => info!("Success"),
                Ok(ReflectSetSuccess::NoChanges) => warn!("Value not changed"),
                Err(err) => error!("{err:?}"),
            }
        });
    }
}

/// System that updates value of the text node to display the value of [`ExampleResource::value`].
fn update_label(mut query: Query<&mut Text, With<ExampleLabel>>, example: Res<ExampleResource>) {
    for mut text in query.iter_mut() {
        text.sections[1].value = format!("{:?}", example.value);
    }
}
