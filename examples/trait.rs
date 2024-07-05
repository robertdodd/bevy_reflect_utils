use bevy::{color::palettes::css, prelude::*};

use bevy_reflect_utils::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_interactions, update_interaction_colors))
        // IMPORTANT: The types you want to operate on must be registered
        .register_type::<DarkButton>()
        .register_type::<LightButton>()
        .run();
}

/// Trait which can be implemented by a component to return a color for an `Interaction` state
#[reflect_trait]
pub trait Interactable {
    fn get_colors(&self, interaction: Interaction) -> InteractionColors;
    fn handle_click(&mut self);
}

// IMPORTANT: Must reflect `Component` and `Interactable`
#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component, Default, Debug, Interactable)]
pub struct LightButton {
    pub selected: bool,
}

impl Interactable for LightButton {
    fn handle_click(&mut self) {
        self.selected = !self.selected;
    }

    fn get_colors(&self, interaction: Interaction) -> InteractionColors {
        match (self.selected, interaction) {
            (true, _) => InteractionColors::new(css::ORANGE_RED, css::WHITE),
            (_, Interaction::Pressed) => InteractionColors::new(css::GRAY, css::BLACK),
            (_, Interaction::Hovered) => InteractionColors::new(css::YELLOW, css::BLACK),
            (_, Interaction::None) => InteractionColors::new(css::ANTIQUE_WHITE, css::BLACK),
        }
    }
}

// IMPORTANT: Must reflect `Component` and `Interactable`
#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component, Default, Debug, Interactable)]
pub struct DarkButton {
    pub selected: bool,
}

impl Interactable for DarkButton {
    fn handle_click(&mut self) {
        self.selected = !self.selected;
    }

    fn get_colors(&self, interaction: Interaction) -> InteractionColors {
        match (self.selected, interaction) {
            (true, _) => InteractionColors::new(css::ORANGE_RED, css::WHITE),
            (_, Interaction::Pressed) => InteractionColors::new(css::BLACK, css::WHITE),
            (_, Interaction::Hovered) => InteractionColors::new(css::GRAY, css::WHITE),
            (_, Interaction::None) => InteractionColors::new(css::DARK_GRAY, css::WHITE),
        }
    }
}

/// Component that controls the colors of an interactable node.
#[derive(Component, Reflect, Debug, Default, Clone)]
#[reflect(Component, Default, Debug)]
pub struct InteractionColors {
    background: Color,
    border: Color,
    text: Color,
}

impl InteractionColors {
    pub fn new(background: impl Into<Color>, border_and_text: impl Into<Color>) -> Self {
        let border_and_text = border_and_text.into();
        Self {
            background: background.into(),
            border: border_and_text,
            text: border_and_text,
        }
    }
}

/// System that spawns a camera and the UI widgets.
fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    root_full_screen_centered_widget(&mut commands, |p| {
        interactable_button_widget(p, "Dark Button", DarkButton::default());
        interactable_button_widget(p, "Light Button", LightButton::default());
        button_widget(
            p,
            "Not Interactable",
            (),
            css::TURQUOISE,
            css::WHITE,
            css::BLACK,
        );

        // Spawn a button that contains both trait components
        button_widget(
            p,
            "Both Traits",
            (DarkButton::default(), LightButton::default()),
            css::TURQUOISE,
            css::WHITE,
            css::BLACK,
        );
    });
}

/// System that handles [`Interaction`] changes.
///
/// Uses a `Command` to try find a component with the `ReflectInteractable` trait to handle click events and update
/// the colors.
fn handle_interactions(
    mut commands: Commands,
    query: Query<(Entity, &Interaction), Changed<Interaction>>,
) {
    for (entity, interaction) in query.iter() {
        let interaction = *interaction;

        // Add command to update colors
        commands.add(move |world: &mut World| {
            // Handle clicks before updating the colors
            if interaction == Interaction::Pressed {
                let mut update_count = 0; // used to assert that the closure wasn't called more than once
                let result = reflect_trait_mut_expect_once::<ReflectInteractable>(
                    world,
                    entity,
                    |reflect_value, reflect_trait| {
                        if let Some(get_interaction_colors) = reflect_trait.get_mut(reflect_value) {
                            get_interaction_colors.handle_click();
                            update_count += 1;
                        }
                    },
                );
                // Handle the result
                match result {
                    Ok(_) => info!("successfull handled click via trait"),
                    Err(err) => error!("error handling click via trait: {:?}", err),
                }
                // Assert that the closure did not run over more than one component
                assert!(update_count <= 1);
            }

            // read the colors from the trait
            let mut read_count = 0; // used to assert that the closure wasn't called more than once
            let result = reflect_trait_find_one::<ReflectInteractable, _>(
                world,
                entity,
                |reflect_value, reflect_trait| {
                    reflect_trait.get(reflect_value).map(|r| {
                        read_count += 1;
                        r.get_colors(interaction)
                    })
                },
            );
            // Handle the result
            match result {
                Ok(Some(colors)) => {
                    if let Some(mut entity_mut) = world.get_entity_mut(entity) {
                        entity_mut.insert(colors);
                    }
                }
                Ok(None) => {
                    warn!("reading colors via trait returned None");
                }
                Err(err) => {
                    error!("error handling click via trait: {:?}", err);
                }
            }
            // Assert that the closure did not run over more than one component
            assert!(read_count <= 1);
        });
    }
}

/// System that updates the colors of a node when the [`InteractionColors`] component changes
fn update_interaction_colors(
    mut query: Query<
        (
            &InteractionColors,
            Option<&Children>,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<InteractionColors>,
    >,
    mut text_query: Query<&mut Text>,
) {
    for (colors, children, mut background_color, mut border_color) in query.iter_mut() {
        // Update background and border color
        *background_color = colors.background.into();
        *border_color = colors.border.into();

        // Update text color of children
        if let Some(children) = children {
            for &child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    for section in text.sections.iter_mut() {
                        section.style.color = colors.text;
                    }
                }
            }
        }
    }
}

/// Utility that spawns a full-screen centered column node.
fn root_full_screen_centered_widget(
    commands: &mut Commands,
    children: impl FnOnce(&mut ChildBuilder),
) {
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
        .with_children(children);
}

/// Utility that spawns an interactable button widget.
fn interactable_button_widget(
    parent: &mut ChildBuilder,
    value: impl Into<String>,
    button: impl Component + Interactable,
) {
    let colors = button.get_colors(Interaction::None);
    button_widget(
        parent,
        value,
        (button, colors.clone()),
        colors.background,
        colors.border,
        colors.text,
    )
}

/// Utility that spawns a button widget.
fn button_widget(
    parent: &mut ChildBuilder,
    value: impl Into<String>,
    extras: impl Bundle,
    background_color: impl Into<BackgroundColor>,
    border_color: impl Into<BorderColor>,
    text_color: impl Into<Color>,
) {
    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    padding: UiRect::all(Val::Px(10.)),
                    border: UiRect::all(Val::Px(1.)),
                    margin: UiRect::bottom(Val::Px(10.)),
                    min_width: Val::Px(250.),
                    ..default()
                },
                background_color: background_color.into(),
                border_color: border_color.into(),
                ..default()
            },
            extras,
        ))
        .with_children(|p| {
            p.spawn(TextBundle::from_section(
                value,
                TextStyle {
                    color: text_color.into(),
                    ..default()
                },
            ));
        });
}
