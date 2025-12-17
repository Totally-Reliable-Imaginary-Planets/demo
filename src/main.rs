use bevy::prelude::*;
mod explorer;
mod galaxy_event;
mod orchestrator;
mod planet;
mod resources;
mod settings;
//mod simulation;
mod simulation_better;
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
        .add_plugins((
            settings::settings_plugin,
            simulation_better::simulation_better_plugin,
        ))
        .run();
}

// Enum that will be used as a global state for the game
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Settings,
    Playing,
}

#[derive(Component)]
pub struct LogScreen;
#[derive(Component)]
pub struct LogText;

#[derive(Component)]
pub struct PlanetAlphaState;
#[derive(Component)]
pub struct PlanetCell;
#[derive(Component)]
pub struct PlanetRocket;
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
}
