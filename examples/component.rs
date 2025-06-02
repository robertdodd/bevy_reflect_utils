use bevy::{
    color::palettes::tailwind,
    ecs::{relationship::RelatedSpawner, spawn::SpawnWith},
    prelude::{Val::*, *},
};

use bevy_reflect_utils::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (spawn_form_controls, update_reflect_labels))
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

/// Component marking the layout node. New controls nodes get inserted here.
#[derive(Component)]
struct Layout;

/// Component added to a `Text` node, that will display the `i32` value of the reflected field.
#[derive(Component, Clone)]
struct ReflectLabelI32(ReflectTarget);

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Spawn 2 entities by default
    // This will trigger 2 control nodes for each of the entities to spawn via the `spawn_form_controls` system.
    commands.spawn(ExampleComponent { value: 1 });
    commands.spawn(ExampleComponent { value: 2 });

    // Spawn the layout node
    commands.spawn((ui_root(), Layout));
}

/// System that spawns a control panel whenever a new `ExampleComponent` is added.
fn spawn_form_controls(
    mut commands: Commands,
    query: Query<Entity, Added<ExampleComponent>>,
    layout_query: Query<Entity, With<Layout>>,
) {
    for entity in query.iter() {
        let layout = layout_query.single().unwrap();
        let target = ReflectTarget::new_component::<ExampleComponent>(entity, "value");
        commands.entity(layout).with_child((
            panel_widget(),
            children![
                title_widget(format!("Entity {:?}", entity)),
                (
                    form_button_grid_widget(),
                    // NOTE: we spawn children using `Children::spawn` so that we can add observers to the
                    // buttons.
                    Children::spawn(SpawnWith(move |p: &mut RelatedSpawner<ChildOf>| {
                        // left button
                        p.spawn(button_widget("<")).observe(on_button_click(
                            target.clone(),
                            -1,
                            Some(0),
                            Some(10),
                        ));
                        // center label
                        p.spawn((
                            Node {
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            children![(Text::default(), ReflectLabelI32(target.clone()))],
                        ));
                        // right button
                        p.spawn(button_widget(">")).observe(on_button_click(
                            target.clone(),
                            1,
                            Some(0),
                            Some(10),
                        ));
                    })),
                ),
            ],
        ));
    }
}

// TODO: Run this system once when a control is added, and once whenever oncce of the buttons is clicked.
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

/// Returns an observer system that increments a `ReflectTarget` by a specified amount and constraints when a
/// `Pointer<Click>` event is triggered.
fn on_button_click(
    target: ReflectTarget,
    amount: i32,
    min: Option<i32>,
    max: Option<i32>,
) -> impl Fn(Trigger<Pointer<Click>>, Commands) {
    move |_trigger, mut commands| {
        let target = target.clone();
        commands.queue(move |world: &mut World| {
            if let Ok(value) = target.read_value::<i32>(world) {
                let mut new_value = value + amount;
                if let Some(min) = min {
                    new_value = new_value.max(min);
                }
                if let Some(max) = max {
                    new_value = new_value.min(max);
                }
                match target.set_value(world, new_value) {
                    Ok(ReflectSetSuccess::Changed) => info!("Success. Value changed."),
                    Ok(ReflectSetSuccess::NoChanges) => warn!("Value not changed."),
                    Err(err) => error!("Set value failed: {err:?}"),
                }
            }
        });
    }
}

fn ui_root() -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Row,
            width: Percent(100.),
            height: Percent(100.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: Px(10.),
            ..default()
        },
        BackgroundColor(tailwind::SLATE_800.into()),
    )
}

fn panel_widget() -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Px(10.)),
            min_width: Px(200.),
            row_gap: Px(10.),
            ..default()
        },
        BackgroundColor(tailwind::SLATE_700.into()),
        BorderRadius::all(Px(10.)),
    )
}

fn title_widget<T: Into<String>>(value: T) -> impl Bundle {
    (
        Node {
            width: Percent(100.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![Text::new(value)],
    )
}

fn form_button_grid_widget() -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Row,
            width: Percent(100.),
            display: Display::Grid,
            padding: UiRect::all(Px(10.)),
            grid_template_columns: vec![
                RepeatedGridTrack::auto(1),
                RepeatedGridTrack::fr(1, 1.),
                RepeatedGridTrack::auto(1),
            ],
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        BackgroundColor(tailwind::SLATE_800.into()),
        BorderRadius::all(Px(10.)),
    )
}

fn button_widget<T: Into<String>>(value: T) -> impl Bundle {
    (
        Button,
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            padding: UiRect::all(Px(4.)),
            min_width: Px(40.),
            ..default()
        },
        BackgroundColor(tailwind::RED_500.into()),
        BorderRadius::all(Px(8.)),
        BoxShadow::new(
            Color::BLACK.with_alpha(0.8),
            Px(0.),
            Px(8.),
            Val::Px(-8.),
            Val::Px(1.),
        ),
        children![Text::new(value)],
    )
}
