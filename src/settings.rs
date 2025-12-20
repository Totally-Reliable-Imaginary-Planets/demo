use super::GameState;
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
        DespawnOnExit(GameState::Settings),
        BackgroundColor(Color::BLACK.with_alpha(0.7)),
        Text::new("Press R to restart"),
        TextFont::default().with_font_size(16.0),
        TextColor(Color::WHITE),
        SettingsDialog,
    ));
}

fn reset_game(
    dialog: Single<Entity, With<SettingsDialog>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    if keyboard_input.pressed(KeyCode::KeyR) {
    next_state.set(GameState::Playing);
    }

    if keyboard_input.pressed(KeyCode::KeyC) {
    next_state.set(GameState::Creative);
    }
}
