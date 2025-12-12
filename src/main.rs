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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    commands.spawn((
        Camera2d,
        Camera::default(),
        Transform::from_xyz(0.0, 0.0, 1000.0),
    ));

    let spacing = 2.0;
    let padding = 12.0;

    // Planet states
    commands.spawn(create_planet_state(
        &asset_server,
        "Planet Alpha",
        Val::Auto,
        Val::Percent(spacing),
        Val::Percent(spacing),
        PlanetAlphaState,
        PlanetAlphaCell,
        PlanetAlphaRocket,
    ));

    commands.spawn(create_planet_state(
        &asset_server,
        "Planet Beta",
        Val::Percent(spacing),
        Val::Auto,
        Val::Percent(spacing),
        PlanetBetaState,
        PlanetBetaCell,
        PlanetBetaRocket,
    ));

    // Log screen
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(0.0),
            left: Val::Percent(2.5),
            width: Val::Percent(95.0),
            height: Val::Percent(30.0),
            padding: UiRect::all(Val::Px(padding)),
            overflow: Overflow::scroll_y(),
            ..default()
        },
        Visibility::Visible,
        theme::background_color(),
        children![(
            Text::new(""),
            theme::basic_font(&asset_server),
            theme::text_color(),
            LogText,
        )],
    ));

    commands.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::SpaceBetween,
            bottom: Val::Percent(35.0),
            left: Val::Percent(35.0),
            width: Val::Percent(30.0),
            min_width: Val::Px(400.0),
            height: Val::Percent(60.0),
            ..default()
        },
        LandedPlanetDialog,
        Visibility::Hidden,
        theme::background_color(),
        children![
            (
                Text::new("What would you do on this planet?"),
                theme::title_font(&asset_server),
                theme::text_color(),
            ),
            create_button(
                &asset_server,
                "Supported Resource",
                SupportedResourceButton,
                Val::Percent(90.0),
                Val::Percent(15.0),
                Val::Percent(5.0),
                Val::Px(5.0),
                Color::WHITE
            ),
            create_button(
                &asset_server,
                "Extract Resource",
                ExtractResourceButton,
                Val::Percent(90.0),
                Val::Percent(15.0),
                Val::Percent(5.0),
                Val::Px(5.0),
                Color::WHITE
            ),
            create_button(
                &asset_server,
                "Available Energy Cell",
                AvailableEnergyCellButton,
                Val::Percent(90.0),
                Val::Percent(15.0),
                Val::Percent(5.0),
                Val::Px(5.0),
                Color::WHITE
            ),
            create_button(
                &asset_server,
                "Take off",
                TakeOffPlanetButton,
                Val::Percent(90.0),
                Val::Percent(15.0),
                Val::Percent(5.0),
                Val::Px(5.0),
                Color::WHITE
            ),
        ],
    ));

    // Spawn land_on_planet_dialog UI
    commands.spawn(land_on_planet_dialog(&asset_server));
}

fn create_button(
    asset_server: &Res<AssetServer>,
    text: &str,
    button_component: impl Component,
    width: Val,
    height: Val,
    left: Val,
    border_stroke: Val,
    border_color: Color,
) -> impl Bundle {
    (
        Button,
        button_component,
        Node {
            width: width,
            height: height,
            left: left,
            border: UiRect::all(border_stroke),
            // horizontally center child text
            justify_content: JustifyContent::Center,
            // vertically center child text
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor::all(border_color),
        children![(
            Text::new(text),
            theme::title_font(asset_server),
            theme::text_color(),
        )],
    )
}

fn create_planet_state(
    asset_server: &Res<AssetServer>,
    planet_name: &str,
    right: Val,
    left: Val,
    top: Val,
    state: impl Component,
    cell: impl Component,
    rocket: impl Component,
) -> impl Bundle {
    let padding = 12.0;
    let width = 10.0;
    let max_width = width + 5.0;
    let height = 15.0;

    (
        Node {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            top: top,
            right: right,
            left: left,
            padding: UiRect::all(Val::Px(padding)),
            width: Val::Percent(width),
            max_width: Val::Percent(max_width),
            height: Val::Percent(height),
            ..default()
        },
        state,
        Visibility::Visible,
        theme::background_color(),
        children![
            (
                Text::new(planet_name),
                theme::title_font(asset_server),
                theme::text_color(),
            ),
            (
                Text::new("Energy cell:"),
                theme::basic_font(asset_server),
                theme::text_color(),
            ),
            (
                Text::new(""),
                theme::basic_font(asset_server),
                theme::text_color(),
                cell
            ),
            (
                Text::new("Rocket:"),
                theme::basic_font(asset_server),
                theme::text_color(),
            ),
            (
                Text::new(""),
                theme::basic_font(asset_server),
                theme::text_color(),
                rocket,
            )
        ],
    )
}

fn land_on_planet_dialog(asset_server: &Res<AssetServer>) -> impl Bundle {
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
                    theme::title_font(asset_server),
                    theme::text_color(),
                ),
                (
                    Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    children![
                        create_button(
                            &asset_server,
                            "Yes",
                            YesButton,
                            Val::Percent(40.0),
                            Val::Percent(100.0),
                            Val::Auto,
                            Val::Px(5.0),
                            Color::WHITE
                        ),
                        create_button(
                            &asset_server,
                            "No",
                            NoButton,
                            Val::Percent(40.0),
                            Val::Percent(100.0),
                            Val::Auto,
                            Val::Px(5.0),
                            Color::WHITE
                        ),
                    ],
                )
            ],
        )],
    )
}
