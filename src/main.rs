use bevy::prelude::*;
mod explorer;
mod galaxy_event;
mod planet;
mod resources;
mod settings;
mod simulation;
mod theme;

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
pub struct PlanetAlphaState;
#[derive(Component)]
pub struct PlanetAlphaCell;
#[derive(Component)]
pub struct PlanetAlphaRocket;
#[derive(Component)]
pub struct PlanetBetaState;
#[derive(Component)]
pub struct PlanetBetaCell;
#[derive(Component)]
pub struct PlanetBetaRocket;

#[derive(Component)]
struct PlanetDialog;
#[derive(Component)]
struct LandedPlanetDialog;

// Marker components for buttons
#[derive(Component)]
struct YesButton;

#[derive(Component)]
struct NoButton;

#[derive(Component)]
struct SupportedResourceButton;

#[derive(Component)]
struct ExtractResourceButton;

#[derive(Component)]
struct AvailableEnergyCellButton;

#[derive(Component)]
struct TakeOffPlanetButton;

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn((
        Camera2d,
        Camera::default(),
        Transform::from_xyz(0.0, 0.0, 1000.0),
    ));

    let spacing = 20.0;
    let padding = 12.0;
    let width = 20.0;
    let max_width = width + 5.0;
    let height = 30.0;

    // Planet states
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            top: Val::Px(spacing),
            left: Val::Px(spacing),
            padding: UiRect::all(Val::Px(padding)),
            width: Val::Percent(width),
            max_width: Val::Percent(max_width),
            height: Val::Percent(height),
            ..default()
        },
        PlanetAlphaState,
        Visibility::Visible,
        theme::background_color(),
        children![
            (
                Text::new("Planet Alpha"),
                theme::title_font(),
                theme::text_color(),
            ),
            (
                Text::new("Charged cell:"),
                theme::basic_font(),
                theme::text_color(),
            ),
            (
                Text::new("..."),
                theme::basic_font(),
                theme::text_color(),
                PlanetAlphaCell,
            ),
            (
                Text::new("Rocket:"),
                theme::basic_font(),
                theme::text_color(),
            ),
            (
                Text::new("0"),
                theme::basic_font(),
                theme::text_color(),
                PlanetAlphaRocket
            )
        ],
    ));

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            top: Val::Px(spacing),
            right: Val::Px(spacing),
            padding: UiRect::all(Val::Px(padding)),
            width: Val::Percent(width),
            max_width: Val::Percent(max_width),
            height: Val::Percent(height),
            ..default()
        },
        PlanetBetaState,
        Visibility::Visible,
        theme::background_color(),
        children![
            (
                Text::new("Planet Beta"),
                theme::title_font(),
                theme::text_color(),
            ),
            (
                Text::new("Charged cell:"),
                theme::basic_font(),
                theme::text_color(),
            ),
            (
                Text::new("..."),
                theme::basic_font(),
                theme::text_color(),
                PlanetBetaCell
            ),
            (
                Text::new("Rocket:"),
                theme::basic_font(),
                theme::text_color(),
            ),
            (
                Text::new("0"),
                theme::basic_font(),
                theme::text_color(),
                PlanetBetaRocket,
            )
        ],
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
        theme::background_color(),
        Text::new(""),
        theme::basic_font(),
        theme::text_color(),
        LogText,
    ));

    commands.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::SpaceBetween,
            bottom: Val::Percent(35.0),
            left: Val::Percent(35.0),
            width: Val::Percent(30.0),
            height: Val::Percent(60.0),
            ..default()
        },
        LandedPlanetDialog,
        Visibility::Hidden,
        theme::background_color(),
        children![
            (Text::new("What would you do on this planet?"),),
            (
                Button,
                SupportedResourceButton,
                Node {
                    width: Val::Percent(90.0),
                    height: Val::Percent(15.0),
                    left: Val::Percent(5.0),
                    border: UiRect::all(px(5)),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor::all(Color::WHITE),
                children![(Text::new("Supported resource"))]
            ),
            (
                Button,
                ExtractResourceButton,
                Node {
                    width: Val::Percent(90.0),
                    height: Val::Percent(15.0),
                    left: Val::Percent(5.0),
                    border: UiRect::all(px(5)),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor::all(Color::WHITE),
                children![(Text::new("Extract resource"))]
            ),
            (
                Button,
                AvailableEnergyCellButton,
                Node {
                    width: Val::Percent(90.0),
                    height: Val::Percent(15.0),
                    left: Val::Percent(5.0),
                    border: UiRect::all(px(5)),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor::all(Color::WHITE),
                children![(Text::new("Available energy cell"))]
            ),
            (
                Button,
                TakeOffPlanetButton,
                Node {
                    width: Val::Percent(90.0),
                    height: Val::Percent(15.0),
                    left: Val::Percent(5.0),
                    border: UiRect::all(px(5)),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor::all(Color::WHITE),
                children![(Text::new("Take off planet"))]
            )
        ],
    ));

    // Spawn land_on_planet_dialog UI
    commands.spawn(land_on_planet_dialog());
}

fn land_on_planet_dialog() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Percent(30.0),
            left: Val::Percent(30.0),
            width: Val::Percent(40.0),
            height: Val::Percent(40.0),
            ..default()
        },
        Visibility::Hidden,
        theme::background_color(),
        PlanetDialog,
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),

                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
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
