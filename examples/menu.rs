//! This example demonstrates an interactive "Settings" menu using reflection for the controls.

use std::slice::Iter;

use bevy::{color::palettes::css, prelude::*, reflect::TypeRegistry};
use serde::Deserialize;

use bevy_reflect_utils::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Settings>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update_reflect_labels,
                update_reflect_visibility,
                handle_i32_click_events,
                handle_enum_click_events,
                handle_serialized_click_events,
                update_preview.run_if(resource_exists_and_changed::<Settings>),
                update_selectable_buttons,
                initialize_selectable_buttons,
                handle_selectable_button_clicked,
            ),
        )
        .register_type::<Settings>()
        .register_type::<Theme>()
        .register_type::<ThemeColor>()
        .run();
}

/// Resource containing data for the settings page UI.
///
/// The reflection controls will operate on this resource.
#[derive(Resource, Reflect, Debug)]
#[reflect(Resource, Default, Debug)]
pub struct Settings {
    show_preview: bool,
    volume: i32,
    theme: Theme,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_preview: true,
            volume: 5,
            theme: Theme::Custom(ThemeColor::Red),
        }
    }
}

/// Component that will update an `i32` value when it is clicked.
#[derive(Component, Clone)]
struct ReflectButtonI32 {
    target: ReflectTarget,
    amount: i32,
    min: Option<i32>,
    max: Option<i32>,
}

/// Component that will toggle between enum variants when clicked.
#[derive(Component)]
struct ReflectButtonEnum {
    target: ReflectTarget,
    direction: EnumDirection,
}

/// Component that will set a serialized value on athe reflect target when clicked.
#[derive(Component, Clone)]
struct ReflectButtonSerialized {
    target: ReflectTarget,
    value: String,
}

/// A custom theme color.
/// IMPORTANT: We must derive and reflect the `Deserialize` trait to set this field via a serialized value.
/// IMPORTANT: We must serive and reflect the `Default` trait so that we can toggle to `Theme::Custom(ThemeColor)`.
#[derive(Debug, Default, PartialEq, Clone, Copy, Reflect, Deserialize)]
#[reflect(Debug, Default, PartialEq, Deserialize)]
enum ThemeColor {
    #[default]
    Red,
    Green,
    Blue,
    Yellow,
    Orange,
    Maroon,
    Turquoise,
}

impl ThemeColor {
    pub fn iter_variants() -> Iter<'static, Self> {
        static THEME_COLORS: [ThemeColor; 7] = [
            ThemeColor::Red,
            ThemeColor::Green,
            ThemeColor::Blue,
            ThemeColor::Yellow,
            ThemeColor::Orange,
            ThemeColor::Maroon,
            ThemeColor::Turquoise,
        ];
        THEME_COLORS.iter()
    }
}

impl From<ThemeColor> for Color {
    fn from(value: ThemeColor) -> Self {
        match value {
            ThemeColor::Red => css::RED,
            ThemeColor::Green => css::GREEN,
            ThemeColor::Blue => css::BLUE,
            ThemeColor::Yellow => css::YELLOW,
            ThemeColor::Orange => css::ORANGE,
            ThemeColor::Maroon => css::MAROON,
            ThemeColor::Turquoise => css::TURQUOISE,
        }
        .into()
    }
}

/// Type describing the settings theme.
/// IMPORTANT: We must serive and reflect the `Default` trait to toggle to the `Custom` variant.
#[derive(Debug, Default, PartialEq, Clone, Copy, Reflect)]
#[reflect(Debug, Default, PartialEq)]
enum Theme {
    #[default]
    Dark,
    Light,
    Custom(ThemeColor),
}

/// Type describing the type that a label points to.
#[derive(Clone)]
enum ReflectLabelKind {
    I32,
    Enum,
    Bool,
}

/// Component added to a `Text` node, that will update the value of the text to the value of the reflected field.
#[derive(Component, Clone)]
struct ReflectLabel {
    target: ReflectTarget,
    kind: ReflectLabelKind,
}

/// Type describing the visibility behavior for a `ReflectUiVisibility` component.
#[derive(Debug, Clone)]
pub enum VisibilityFunc {
    /// Visible when `PartialEq` against the serialized value is `true`
    PartialEqSerialized(String),
    /// Visibility when the field is accessible
    Accessible,
}

/// Component added to a [`Node`] that controls its visibility based on the value of a `ReflectTarget`.
#[derive(Component, Debug, Clone)]
pub struct ReflectUiVisibility {
    pub target: ReflectTarget,
    pub visibility_func: VisibilityFunc,
    /// The visibility when the reflect field is not accessible
    pub default_visibility: bool,
    /// Tracks the current visibility status
    pub is_visible: Option<bool>,
}

/// Component marking the "preview" node.
#[derive(Component)]
struct Preview;

/// Type describing the theme for the `Preview` node.
struct ThemeStyle {
    background_color: Color,
    border_color: Color,
    text_color: Color,
}

impl From<Theme> for ThemeStyle {
    fn from(value: Theme) -> Self {
        match value {
            Theme::Dark => ThemeStyle {
                background_color: Color::BLACK,
                border_color: Color::WHITE,
                text_color: Color::WHITE,
            },
            Theme::Light => ThemeStyle {
                background_color: Color::WHITE,
                border_color: Color::BLACK,
                text_color: Color::BLACK,
            },
            Theme::Custom(color) => match color {
                ThemeColor::Red => ThemeStyle {
                    background_color: color.into(),
                    border_color: Color::WHITE,
                    text_color: Color::WHITE,
                },
                ThemeColor::Green => ThemeStyle {
                    background_color: color.into(),
                    border_color: Color::WHITE,
                    text_color: Color::BLACK,
                },
                ThemeColor::Blue => ThemeStyle {
                    background_color: color.into(),
                    border_color: Color::WHITE,
                    text_color: Color::WHITE,
                },
                ThemeColor::Yellow => ThemeStyle {
                    background_color: color.into(),
                    border_color: Color::WHITE,
                    text_color: Color::BLACK,
                },
                ThemeColor::Orange => ThemeStyle {
                    background_color: color.into(),
                    border_color: Color::WHITE,
                    text_color: Color::WHITE,
                },
                ThemeColor::Maroon => ThemeStyle {
                    background_color: color.into(),
                    border_color: Color::WHITE,
                    text_color: Color::WHITE,
                },
                ThemeColor::Turquoise => ThemeStyle {
                    background_color: color.into(),
                    border_color: Color::WHITE,
                    text_color: Color::BLACK,
                },
            },
        }
    }
}

/// Component marking a selectable button and describing its state.
#[derive(Component, Default)]
pub enum SelectableButton {
    #[default]
    Default,
    Selected,
}

/// System that spawns the UI for this example.
fn setup(mut commands: Commands, settings: Res<Settings>, app_type_registry: Res<AppTypeRegistry>) {
    commands.spawn(Camera2d);

    let type_registry = app_type_registry.read();

    // Spawn a full screen node containing the settings panel centered centered on the screen.
    root_full_screen_centered_panel_widget(&mut commands, |p| {
        title_widget(p, "Settings");

        // Spawn a widget showing a preview of the theme.
        // This node will only be visible when the value of `Settings::show_preview` is `true`
        form_control_widget(
            p,
            "Preview",
            ReflectUiVisibility {
                target: ReflectTarget::new_resource::<Settings>("show_preview"),
                visibility_func: VisibilityFunc::PartialEqSerialized("{\"bool\":true}".to_string()),
                default_visibility: false,
                is_visible: None,
            },
            |p| {
                preview_widget(p, &settings);
            },
        );

        // Spawn a widget controlling `Settings::show_preview`
        let target = ReflectTarget::new_resource::<Settings>("show_preview");
        form_control_widget(p, "Show Preview", (), |p| {
            form_button_grid_widget(p, |p| {
                button_widget(
                    p,
                    "<",
                    // Sets the value to `false` when clicked
                    ReflectButtonSerialized {
                        target: target.clone(),
                        value: "{\"bool\":false}".to_string(),
                    },
                );
                label_widget(
                    p,
                    "",
                    ReflectLabel {
                        target: target.clone(),
                        kind: ReflectLabelKind::Bool,
                    },
                );
                button_widget(
                    p,
                    ">",
                    // Sets the value to `true` when clicked
                    ReflectButtonSerialized {
                        target: target.clone(),
                        value: "{\"bool\":true}".to_string(),
                    },
                );
            });
        });

        // Spawn a widget controlling `Settings::theme`
        let target = ReflectTarget::new_resource::<Settings>("theme");
        form_control_widget(p, "Theme", (), |p| {
            form_button_grid_widget(p, |p| {
                button_widget(
                    p,
                    "<",
                    ReflectButtonEnum {
                        target: target.clone(),
                        direction: EnumDirection::Backward,
                    },
                );
                label_widget(
                    p,
                    "",
                    ReflectLabel {
                        target: target.clone(),
                        kind: ReflectLabelKind::Enum,
                    },
                );
                button_widget(
                    p,
                    ">",
                    ReflectButtonEnum {
                        target: target.clone(),
                        direction: EnumDirection::Forward,
                    },
                );
            });
        });

        // Spawn a widget controlling `Settings::theme.0`, which is only accessible when theme is `Theme::Custom`.
        // The `ReflectUiVisibility` component will hide this node when the target field is not accessible.
        let target = ReflectTarget::new_resource::<Settings>("theme.0");
        form_control_widget(
            p,
            "Theme Custom Color",
            // Only show this node when the reflect target field is accessible.
            ReflectUiVisibility {
                target: target.clone(),
                visibility_func: VisibilityFunc::Accessible,
                default_visibility: false,
                is_visible: None,
            },
            |p| {
                button_grid_widget(p, |p| {
                    for theme_color in ThemeColor::iter_variants() {
                        color_button_widget(p, &type_registry, target.clone(), *theme_color);
                    }
                });
            },
        );

        // Spawn a widget controlling `Settings::volume`
        let target = ReflectTarget::new_resource::<Settings>("volume");
        form_control_widget(p, "Volume", (), |p| {
            form_button_grid_widget(p, |p| {
                button_widget(
                    p,
                    "-",
                    ReflectButtonI32 {
                        target: target.clone(),
                        amount: -1,
                        min: Some(0),
                        max: Some(10),
                    },
                );
                label_widget(
                    p,
                    "",
                    ReflectLabel {
                        target: target.clone(),
                        kind: ReflectLabelKind::I32,
                    },
                );
                button_widget(
                    p,
                    "+",
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
}

/// Exclusive system which updates the text value of `ReflectLabel` components.
fn update_reflect_labels(world: &mut World) {
    let mut query = world.query_filtered::<Entity, With<ReflectLabel>>();
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
            .get::<ReflectLabel>()
            .cloned()
            .unwrap();

        // Get the current value of the field
        let value = match label.kind {
            ReflectLabelKind::Enum => label.target.read_enum_variant_name(world),
            ReflectLabelKind::I32 => label
                .target
                .read_value::<i32>(world)
                .map(|value| format!("{value}")),
            ReflectLabelKind::Bool => {
                label
                    .target
                    .read_value::<bool>(world)
                    .map(|value| match value {
                        true => "Yes".to_string(),
                        false => "No".to_string(),
                    })
            }
        };

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

/// System that handles click events on `ReflectButtonSerialized` components.
///
/// Sets the serialized value via reflection.
fn handle_serialized_click_events(
    mut commands: Commands,
    query: Query<(&ReflectButtonSerialized, &Interaction), Changed<Interaction>>,
) {
    for (button, interaction) in query.iter() {
        if *interaction == Interaction::Pressed {
            let button = button.clone();
            commands.queue(move |world: &mut World| {
                match button.target.set_value_serialized(world, &button.value) {
                    Ok(ReflectSetSuccess::Changed) => info!("Success. Value changed."),
                    Ok(ReflectSetSuccess::NoChanges) => warn!("Value not changed."),
                    Err(err) => error!("Set value failed: {err:?}"),
                }
            });
        }
    }
}

/// System that handles click events on `ReflectButtonEnum` components.
///
/// Toggles the enum variant in the direction specified by `ReflectButtonEnum::direction`.
fn handle_enum_click_events(
    mut commands: Commands,
    query: Query<(&ReflectButtonEnum, &Interaction), Changed<Interaction>>,
) {
    for (button, interaction) in query.iter() {
        if *interaction == Interaction::Pressed {
            let target = button.target.clone();
            let direction = button.direction;
            commands.queue(move |world: &mut World| {
                match target.toggle_reflect_enum(world, direction) {
                    Ok(ReflectSetSuccess::Changed) => info!("Success. Value changed."),
                    Ok(ReflectSetSuccess::NoChanges) => warn!("Value not changed."),
                    Err(err) => error!("Set value failed: {err:?}"),
                }
            });
        }
    }
}

/// Exclusive system that updates the visibility of nodes with a `ReflectUiVisibility` component.
fn update_reflect_visibility(world: &mut World) {
    // TODO: There must be a better way to do this than collecting the query results into a vector.
    let mut query = world.query_filtered::<Entity, With<ReflectUiVisibility>>();
    let entities: Vec<Entity> = query.iter(world).collect();
    if entities.is_empty() {
        return;
    }

    for entity in entities.iter() {
        // Read the `ReflectUiVisibility` component
        // SAFETY: These unwraps should be okay because the query above ensured they have the component
        let reflect_visibility = world
            .get_entity(*entity)
            .unwrap()
            .get::<ReflectUiVisibility>()
            .cloned()
            .unwrap();

        // Read whether the field is visible
        let is_visible = match reflect_visibility.visibility_func {
            VisibilityFunc::PartialEqSerialized(serialized_value) => reflect_visibility
                .target
                .partial_eq_serialized(world, &serialized_value),
            VisibilityFunc::Accessible => Ok(reflect_visibility
                .target
                .read_value_serialized(world)
                .is_ok()),
        }
        .unwrap_or(reflect_visibility.default_visibility);

        if Some(is_visible) != reflect_visibility.is_visible {
            if let Ok(mut entity_mut) = world.get_entity_mut(*entity) {
                // Update the display value
                if let Some(mut node) = entity_mut.get_mut::<Node>() {
                    node.display = match is_visible {
                        true => Display::Flex,
                        false => Display::None,
                    };
                }
                // Update the visibility component
                if let Some(mut visibility) = entity_mut.get_mut::<Visibility>() {
                    *visibility = match is_visible {
                        true => Visibility::Inherited,
                        false => Visibility::Hidden,
                    };
                }
                // Update the visibility flag on the hider component
                if let Some(mut hider) = entity_mut.get_mut::<ReflectUiVisibility>() {
                    hider.is_visible = Some(is_visible);
                }
            }
        }
    }
}

/// System that updates the colors of the `Preview` node whenever the `Settings` resources changes.
fn update_preview(
    settings: Res<Settings>,
    mut query: Query<(&mut BackgroundColor, &mut BorderColor, &Children), With<Preview>>,
    mut text_query: Query<&mut Text>,
    mut text_color_query: Query<&mut TextColor>,
) {
    info!("Settings changed ==> Updating preview");

    for (mut bg, mut border, children) in query.iter_mut() {
        let style: ThemeStyle = settings.theme.into();
        *bg = style.background_color.into();
        *border = style.border_color.into();
        for &child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                text.0 = format!("Volume: {}", settings.volume);
            }
            if let Ok(mut text_color) = text_color_query.get_mut(child) {
                text_color.0 = style.text_color;
            }
        }
    }
}

/// System that updates the colors when the state of a `SelectableButton` changes.
fn update_selectable_buttons(
    mut query: Query<(&SelectableButton, &mut BorderColor), Changed<SelectableButton>>,
) {
    for (button, mut border) in query.iter_mut() {
        *border = match *button {
            SelectableButton::Default => Color::NONE.into(),
            SelectableButton::Selected => Color::WHITE.into(),
        };
    }
}

/// System that updates the selected state of `SelectableButton` components when they are added or their visibility is
/// changed.
///
/// This system, coupled with the `handle_selectable_button_clicked` system, lets us avoid using an exclusive system
/// that updates the selected state each frame.
#[allow(clippy::type_complexity)]
fn initialize_selectable_buttons(
    mut commands: Commands,
    query: Query<
        (Entity, &ReflectButtonSerialized),
        Or<(Added<SelectableButton>, Changed<InheritedVisibility>)>,
    >,
) {
    for (entity, reflect_button) in query.iter() {
        let reflect_button = reflect_button.clone();
        commands.queue(move |world: &mut World| {
            // read whether it is selected
            let is_selected = reflect_button
                .target
                .partial_eq_serialized(world, &reflect_button.value)
                .unwrap_or(false);

            // Update the button state
            if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
                if let Some(mut selectable_button) = entity_mut.get_mut::<SelectableButton>() {
                    *selectable_button = match is_selected {
                        true => SelectableButton::Selected,
                        false => SelectableButton::Default,
                    };
                }
            }
        });
    }
}

/// System that marks `SelectableButton` components as selected when they are clicked, and de-selects other selectable
/// buttons with the same parent.
#[allow(clippy::type_complexity)]
fn handle_selectable_button_clicked(
    query: Query<(Entity, &Interaction, &Parent), (Changed<Interaction>, With<SelectableButton>)>,
    children_query: Query<&Children>,
    mut button_query: Query<&mut SelectableButton>,
) {
    for (entity, interaction, parent) in query.iter() {
        if *interaction == Interaction::Pressed {
            // Iterate over children in the parent node and update their visibility
            if let Ok(children) = children_query.get(parent.get()) {
                for &child in children.iter() {
                    if let Ok(mut button) = button_query.get_mut(child) {
                        *button = match child == entity {
                            true => SelectableButton::Selected,
                            false => SelectableButton::Default,
                        }
                    }
                }
            }
        }
    }
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

fn color_button_widget(
    parent: &mut ChildBuilder,
    type_registry: &TypeRegistry,
    target: ReflectTarget,
    theme_color: ThemeColor,
) {
    // Serialize the value of `theme_color`.
    // You could set it manually, for example: "{\"menu::ThemeColor\": Red}"
    let serialized_value = serialize_reflect_value(type_registry, &theme_color).unwrap();

    let color: Color = theme_color.into();

    parent.spawn((
        Button,
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            padding: UiRect::all(Val::Px(10.)),
            border: UiRect::all(Val::Px(1.)),
            ..default()
        },
        BackgroundColor(color),
        BorderColor(Color::NONE),
        // NOTE: We don't need to know whether it is selected by default, as the `initialize_selectable_buttons` system
        // will set it when the button is added.
        SelectableButton::default(),
        // NOTE: The `ReflectButtonSerialized` component will set the serialized value on the target when clicked.
        ReflectButtonSerialized {
            target: target.clone(),
            value: serialized_value,
        },
    ));
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

fn preview_widget(parent: &mut ChildBuilder, settings: &Settings) {
    let style: ThemeStyle = settings.theme.into();
    parent
        .spawn((
            Node {
                width: Val::Percent(100.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(10.)),
                border: UiRect::all(Val::Px(1.)),
                margin: UiRect::bottom(Val::Px(10.)),
                ..default()
            },
            BackgroundColor(style.background_color),
            BorderColor(style.border_color),
            Preview,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(format!("Volume: {}", settings.volume)),
                TextColor(style.text_color),
            ));
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

fn root_full_screen_centered_widget(
    commands: &mut Commands,
    children: impl FnOnce(&mut ChildBuilder),
) {
    commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(children);
}

fn root_full_screen_centered_panel_widget(
    commands: &mut Commands,
    children: impl FnOnce(&mut ChildBuilder),
) {
    root_full_screen_centered_widget(commands, |p| {
        panel_widget(p, children);
    });
}

pub fn button_grid_widget(parent: &mut ChildBuilder, children: impl FnOnce(&mut ChildBuilder)) {
    let column_count = 4;
    let gap = 10.;

    parent
        .spawn(Node {
            width: Val::Percent(100.),
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::px(column_count, 40.),
            grid_template_rows: RepeatedGridTrack::px(2, 40.),
            column_gap: Val::Px(gap),
            row_gap: Val::Px(gap),
            justify_content: JustifyContent::SpaceBetween,
            align_content: AlignContent::SpaceBetween,
            ..default()
        })
        .with_children(children);
}
