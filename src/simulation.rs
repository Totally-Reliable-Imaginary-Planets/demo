use bevy::prelude::*;

use crate::explorer::Explorer;
use crate::planet::Planet;
use crate::resources::EventSpawnTimer;
use crate::resources::PlanetEntities;

use crate::GameState;
use crate::LogText;
use crate::NoButton;
use crate::PlanetDialog;
use crate::YesButton;

pub fn simulation_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Playing), setup)
        .add_systems(
            Update,
            (
                check_entities_and_end_game,
                crate::explorer::movement::explorer_movement_system_wasd,
                crate::explorer::movement::check_explorer_reach,
                crate::galaxy_event::event_spawner_system,
                crate::galaxy_event::event_handler_system,
                crate::galaxy_event::cleanup_events_system,
                crate::galaxy_event::event_visual_system,
                yes_button_system,
                no_button_system,
            )
                .run_if(in_state(GameState::Playing)),
        );
}

fn setup(mut commands: Commands, mut dialog_query: Query<&mut Visibility, With<LogText>>) {
    // Planets
    let planet1 = commands
        .spawn((
            Sprite {
                color: Color::srgb(0.3, 0.5, 0.8),
                custom_size: Some(Vec2::new(80.0, 80.0)),
                ..default()
            },
            Transform::from_xyz(-300.0, 0.0, 0.0),
            Planet::new("Planet Alpha", Vec2::new(-300.0, 0.0)),
        ))
        .id();

    let planet2 = commands
        .spawn((
            Sprite {
                color: Color::srgb(0.8, 0.3, 0.3),
                custom_size: Some(Vec2::new(80.0, 80.0)),
                ..default()
            },
            Transform::from_xyz(300.0, 0.0, 0.0),
            Planet::new("Planet Beta", Vec2::new(300.0, 0.0)),
        ))
        .id();

    // Explorer
    commands.spawn((
        Sprite {
            color: Color::srgb(0.9, 0.9, 0.1),
            custom_size: Some(Vec2::new(30.0, 30.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
        Explorer::new(Some(planet2), 150.0),
    ));

    // Resources
    commands.insert_resource(EventSpawnTimer(Timer::from_seconds(
        5.0,
        TimerMode::Repeating,
    )));
    commands.insert_resource(PlanetEntities {
        planets: vec![planet1, planet2],
    });

    for mut visibility in &mut dialog_query {
        *visibility = Visibility::Visible;
    }
}

fn check_entities_and_end_game(
    mut commands: Commands,
    planet: Query<&Planet>,
    mut next_state: ResMut<NextState<GameState>>,
    query: Query<Entity, With<Explorer>>,
    mut log_query: Query<&mut Text, With<LogText>>,
    mut dialog_query: Query<&mut Visibility, With<LogText>>,
) {
    if planet.is_empty() {
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

fn yes_button_system(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<YesButton>)>,
    mut explorer_query: Single<&mut Transform, With<Explorer>>,
    mut dialog_query: Query<&mut Visibility, With<PlanetDialog>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            handle_button_press(&mut explorer_query, &mut dialog_query, -300.0, 300.0);
            info!("No button pressed");
        }
    }
}
fn no_button_system(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<NoButton>)>,
    mut explorer_query: Single<&mut Transform, With<Explorer>>,
    mut dialog_query: Query<&mut Visibility, With<PlanetDialog>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            handle_button_press(&mut explorer_query, &mut dialog_query, -230.0, 230.0);
            info!("No button pressed");
        }
    }
}

fn handle_button_press(
    explorer: &mut Transform,
    dialog_query: &mut Query<&mut Visibility, With<PlanetDialog>>,
    left_pos: f32,
    right_pos: f32,
) {
    explorer.translation.x = if explorer.translation.x < 0.0 {
        left_pos
    } else {
        right_pos
    };

    for mut visibility in dialog_query {
        *visibility = Visibility::Hidden;
    }
}
