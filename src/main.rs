use bevy::prelude::*;

mod explorer;
mod galaxy_event;
mod planet;
mod resources;
mod simulation;

use crate::explorer::Explorer;
use crate::planet::Planet;
use crate::resources::EventSpawnTimer;
use crate::resources::PlanetEntities;
use crate::simulation::LogText;
use crate::simulation::PlanetDialog;

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
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Percent(2.5),
                width: Val::Percent(95.0),
                height: Val::Percent(30.0),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.7)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont::default().with_font_size(16.0),
                TextColor(Color::WHITE),
                LogText,
            ));
        });
}

mod settings {
    use super::GameState;
    use crate::LogText;
    use bevy::prelude::*;

    #[derive(Component)]
    struct SettingsDialog;

    pub fn settings_plugin(app: &mut App) {
        app.add_systems(OnEnter(GameState::Settings), setup)
            .add_systems(Update, (reset_game.run_if(in_state(GameState::Settings)),));
    }

    fn setup(mut commands: Commands) {
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Percent(2.5),
                left: Val::Percent(2.5),
                width: Val::Percent(95.0),
                height: Val::Percent(95.0),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.7)),
            Text::new("Press R to restart"),
            TextFont::default().with_font_size(16.0),
            TextColor(Color::WHITE),
            SettingsDialog,
        ));
    }

    fn reset_game(
        mut commands: Commands,
        dialog: Single<Entity, With<SettingsDialog>>,
        keyboard_input: Res<ButtonInput<KeyCode>>,
        mut next_state: ResMut<NextState<GameState>>,
        mut log_query: Query<&mut Text, With<LogText>>,
    ) {
        if !keyboard_input.pressed(KeyCode::KeyR) {
            return;
        }
        next_state.set(GameState::Playing);
        commands.entity(*dialog).despawn();

        // Update UI text instead of printing
        if let Ok(mut text) = log_query.single_mut() {
            text.0 = String::new();
        }
    }
}
