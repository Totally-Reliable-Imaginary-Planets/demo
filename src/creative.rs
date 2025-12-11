use super::GameState;
use bevy::prelude::*;

#[derive(Component)]
struct SettingsDialog;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Creative), setup)
        .add_systems(Update, (check_entities_and_end_game).run_if(in_state(GameState::Creative)),);
}

fn setup(mut commands: Commands,
    mut dialog_query: Query<
        &mut Visibility,
        Or<(With<LogText>, With<PlanetAlphaState>)>,
    >,) {
    let orchestrator = Orchestrator::new();
    let explorer_handl = ExplorerHandler::new();

    // Planets
    let mut p1 = trip::trip(
        0,
        orchestrator.orch_rx_p1.clone(),
        orchestrator.planet_tx_p1.clone(),
        explorer_handl.expl_rx_p1.clone(),
    )
    .expect("Error createing planet1");
    let planet1 = commands
        .spawn((
            Sprite {
                image: asset_server.load("sprites/Terran.png"),
                custom_size: Some(Vec2::new(120.0, 120.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
            Planet::new("Alpha", Vec2::new(0.0, 0.0)),
            PlanetAlpha,
        ))
        .id();

    for mut visibility in &mut dialog_query {
        *visibility = Visibility::Visible;
    }

    commands.spawn(());
}


fn check_entities_and_end_game(
    mut commands: Commands,
    planet: Query<&Planet>,
    explorer: Query<&Planet>,
    mut next_state: ResMut<NextState<GameState>>,
    query: Query<Entity, With<Planet>>,
    mut log_query: Query<&mut Text, With<LogText>>,
    mut dialog_query: Query<
        &mut Visibility,
        Or<(With<LogText>, With<PlanetAlphaState>)>,
    >,
) {
    if planet.is_empty() || explorer.is_empty() {
        for mut visibility in &mut dialog_query {
            *visibility = Visibility::Hidden;
        }
        for entity in &query {
            commands.entity(entity).despawn();
        }
        // No player entity found → end game
        next_state.set(GameState::Settings);
        // Update UI text instead of printing
        if let Ok(mut text) = log_query.single_mut() {
            text.0 = String::new();
        }
    }
}
