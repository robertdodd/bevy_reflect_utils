use bevy::{color::palettes::css, prelude::*};

use bevy_reflect_utils::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                setup_new_example_components,
                update_reflect_labels,
                handle_i32_click_events,
            ),
        )
        // IMPORTANT: The types you want to operate on must be registered
        .register_type::<ExampleComponent>()
        .run();
}

// IMPORTANT: The types you operate on must derive `Reflect`
#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component, Default, Debug)]
pub struct ExampleComponent {
    value: i32,
}

/// Component marking the layout node. New panels get inserted here.
#[derive(Component)]
struct Layout;

/// Component added to a `Text` node, that will display the `i32` value of the reflected field.
#[derive(Component, Clone)]
struct ReflectLabelI32(ReflectTarget);

/// Component that will update an `i32` value when it is clicked.
#[derive(Component, Clone)]
struct ReflectButtonI32 {
    target: ReflectTarget,
    amount: i32,
    min: Option<i32>,
    max: Option<i32>,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Spawn 2 entities by default
    commands.spawn(ExampleComponent { value: 1 });
    commands.spawn(ExampleComponent { value: 2 });

    // Spawn the layout node
    root_full_screen_centered_widget(&mut commands, Layout);
}

fn setup_new_example_components(
    mut commands: Commands,
    query: Query<Entity, Added<ExampleComponent>>,
    layout_query: Query<Entity, With<Layout>>,
) {
    for entity in query.iter() {
        let layout = layout_query.single();
        commands.entity(layout).with_children(|p| {
            panel_widget(p, |p| {
                title_widget(p, format!("Entity {:?}", entity));

                // Spawn a widget controlling `Settings::show_preview`
                let target = ReflectTarget::new_component::<ExampleComponent>(entity, "value");
                form_control_widget(p, "Value", (), |p| {
                    form_button_grid_widget(p, |p| {
                        button_widget(
                            p,
                            "<",
                            ReflectButtonI32 {
                                target: target.clone(),
                                amount: -1,
                                min: Some(0),
                                max: Some(10),
                            },
                        );
                        label_widget(p, "", ReflectLabelI32(target.clone()));
                        button_widget(
                            p,
                            ">",
                            ReflectButtonI32 {
                                target: target.clone(),
                                amount: 1,
                                min: Some(0),
                                max: Some(10),
                            },
                        );
                    });
                });
            });
        });
    }
}

/// Exclusive system which updates the text value of `ReflectLabel` components.
fn update_reflect_labels(world: &mut World) {
    let mut query = world.query_filtered::<Entity, With<ReflectLabelI32>>();
    let entities: Vec<Entity> = query.iter(world).collect();
    if entities.is_empty() {
        return;
    }

    for entity in entities.iter() {
        // Read the label component
        // SAFETY: These unwraps should be okay because the query ensured they have the component
        let label = world
            .get_entity(*entity)
            .unwrap()
            .get::<ReflectLabelI32>()
            .cloned()
            .unwrap();

        // Get the current value of the field
        let value = label
            .0
            .read_value::<i32>(world)
            .map(|value| format!("{value}"));

        // Update the label text
        if let Ok(mut entity_ref) = world.get_entity_mut(*entity) {
            if let Some(mut text) = entity_ref.get_mut::<Text>() {
                text.0 = value.unwrap_or("N/A".to_string());
            }
        }
    }
}

/// System that handles click events on `ReflectButtonI32` components.
fn handle_i32_click_events(
    mut commands: Commands,
    query: Query<(&ReflectButtonI32, &Interaction), Changed<Interaction>>,
) {
    for (button, interaction) in query.iter() {
        if *interaction == Interaction::Pressed {
            // Clone the button so we can move it to the command
            let button = button.clone();

            commands.queue(move |world: &mut World| {
                if let Ok(value) = button.target.read_value::<i32>(world) {
                    let mut new_value = value + button.amount;
                    if let Some(min) = button.min {
                        new_value = new_value.max(min);
                    }
                    if let Some(max) = button.max {
                        new_value = new_value.min(max);
                    }
                    match button.target.set_value(world, new_value) {
                        Ok(ReflectSetSuccess::Changed) => info!("Success. Value changed."),
                        Ok(ReflectSetSuccess::NoChanges) => warn!("Value not changed."),
                        Err(err) => error!("Set value failed: {err:?}"),
                    }
                }
            });
        }
    }
}

fn root_full_screen_centered_widget(commands: &mut Commands, extras: impl Bundle) {
    commands.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        extras,
    ));
}

fn panel_widget(parent: &mut ChildBuilder, children: impl FnOnce(&mut ChildBuilder)) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.)),
                border: UiRect::all(Val::Px(1.)),
                min_width: Val::Px(200.),
                ..default()
            },
            BorderColor(Color::WHITE),
        ))
        .with_children(children);
}

fn title_widget(parent: &mut ChildBuilder, value: impl Into<String>) {
    parent
        .spawn(Node {
            width: Val::Percent(100.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            margin: UiRect::bottom(Val::Px(10.)),
            ..default()
        })
        .with_children(|p| {
            text_widget(p, value, ());
        });
}

fn text_widget(parent: &mut ChildBuilder, value: impl Into<String>, extras: impl Bundle) {
    parent.spawn((Text::new(value), extras));
}

fn form_control_widget(
    parent: &mut ChildBuilder,
    label: impl Into<String>,
    extras: impl Bundle,
    children: impl FnOnce(&mut ChildBuilder),
) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.),
                margin: UiRect::bottom(Val::Px(10.)),
                ..default()
            },
            extras,
        ))
        .with_children(|p| {
            form_label_widget(p, label);
            children(p);
        });
}

fn form_label_widget(parent: &mut ChildBuilder, label: impl Into<String>) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            width: Val::Percent(100.),
            margin: UiRect::bottom(Val::Px(10.)),
            ..default()
        })
        .with_children(|p| {
            text_widget(p, label, ());
        });
}

fn form_button_grid_widget(parent: &mut ChildBuilder, children: impl FnOnce(&mut ChildBuilder)) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            width: Val::Percent(100.),
            display: Display::Grid,
            grid_template_columns: vec![
                RepeatedGridTrack::auto(1),
                RepeatedGridTrack::fr(1, 1.),
                RepeatedGridTrack::auto(1),
            ],
            grid_template_rows: RepeatedGridTrack::min_content(1),
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .with_children(children);
}

fn button_widget(parent: &mut ChildBuilder, value: impl Into<String>, extras: impl Bundle) {
    parent
        .spawn((
            Button,
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(10.)),
                ..default()
            },
            BackgroundColor(css::GRAY.into()),
            extras,
        ))
        .with_children(|p| {
            text_widget(p, value, ());
        });
}

fn label_widget(parent: &mut ChildBuilder, value: impl Into<String>, extras: impl Bundle) {
    parent
        .spawn(Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|p| {
            text_widget(p, value, extras);
        });
}
