use bevy::prelude::*;

mod explorer;
mod galaxy_event;
mod planet;
mod resources;
mod settings;
mod simulation;

use crate::explorer::Explorer;
use crate::planet::Planet;
use crate::resources::EventSpawnTimer;
use crate::resources::PlanetEntities;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        .add_plugins((simulation::simulation_plugin, settings::settings_plugin))
        .run();
}

// Enum that will be used as a global state for the game
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Playing,
    Settings,
}

#[derive(Component)]
pub struct LogScreen;
#[derive(Component)]
pub struct LogText;

#[derive(Component)]
struct PlanetDialog;

// Marker components for buttons
#[derive(Component)]
struct YesButton;

#[derive(Component)]
struct NoButton;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn((
        Camera2d,
        Camera::default(),
        Transform::from_xyz(0.0, 0.0, 1000.0),
    ));

    // Log screen
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(0.0),
            left: Val::Percent(2.5),
            width: Val::Percent(95.0),
            height: Val::Percent(30.0),
            overflow: Overflow::scroll_y(),
            ..default()
        },
        Visibility::Visible,
        BackgroundColor(Color::BLACK.with_alpha(0.7)),
        Text::new(""),
        TextFont::default().with_font_size(16.0),
        TextColor(Color::WHITE),
        LogText,
    ));

    // Spawn dialog UI
    commands.spawn(dialog());
}

fn dialog() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Percent(35.0),
            left: Val::Percent(35.0),
            width: Val::Percent(40.0),
            height: Val::Percent(40.0),
            ..default()
        },
        Visibility::Hidden,
        BackgroundColor(Color::BLACK.with_alpha(0.7)),
        PlanetDialog,
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),

                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.7)),
            children![
                (
                    Text::new("You have reached a planet do you want to land on it?"),
                    TextFont::default().with_font_size(16.0),
                    TextColor(Color::WHITE),
                ),
                (
                    Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    children![
                        (
                            Button,
                            Node {
                                width: px(150),
                                height: px(65),
                                border: UiRect::all(px(5)),
                                // horizontally center child text
                                justify_content: JustifyContent::Center,
                                // vertically center child text
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BorderColor::all(Color::WHITE),
                            YesButton,
                            children![(
                                Text::new("Yes"),
                                TextFont::default().with_font_size(16.0),
                                TextColor(Color::WHITE),
                            )]
                        ),
                        (
                            Button,
                            Node {
                                width: px(150),
                                height: px(65),
                                border: UiRect::all(px(5)),
                                // horizontally center child text
                                justify_content: JustifyContent::Center,
                                // vertically center child text
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BorderColor::all(Color::WHITE),
                            NoButton,
                            children![(
                                Text::new("No"),
                                TextFont::default().with_font_size(16.0),
                                TextColor(Color::WHITE),
                            )],
                        )
                    ],
                )
            ],
        )],
    )
}
