use bevy::prelude::*;

use crate::explorer::Explorer;
use crate::explorer::Landed;
use crate::explorer::Roaming;
use crate::planet::Planet;
use crate::resources::EventSpawnTimer;
use crate::resources::PlanetEntities;

use crate::GameState;
use crate::LandedPlanetDialog;
use crate::LogText;
use crate::NoButton;
use crate::PlanetDialog;
use crate::TakeOffPlanetButton;
use crate::YesButton;

pub fn simulation_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Playing), setup)
        .add_systems(
            Update,
            (
                check_entities_and_end_game,
                landed_dialog_visibility,
                crate::explorer::movement::explorer_movement_system_wasd,
                crate::explorer::movement::check_explorer_reach,
                crate::galaxy_event::event_spawner_system,
                crate::galaxy_event::event_handler_system,
                crate::galaxy_event::cleanup_events_system,
                crate::galaxy_event::event_visual_system,
                yes_button_system,
                no_button_system,
                take_off_button_system,
            )
                .run_if(in_state(GameState::Playing)),
        );
}

#[derive(Component)]
struct PlanetAlpha;
#[derive(Component)]
struct PlanetBeta;

fn setup(mut commands: Commands, mut dialog_query: Query<&mut Visibility, With<LogText>>) {
    // Planets
    let planet1 = commands
        .spawn((
            Sprite {
                color: Color::srgb(0.3, 0.5, 0.8),
                custom_size: Some(Vec2::new(80.0, 80.0)),
                ..default()
            },
            Transform::from_xyz(-400.0, 0.0, 0.0),
            Planet::new("Planet Alpha", Vec2::new(-400.0, 0.0)),
            PlanetAlpha,
        ))
        .id();

    let planet2 = commands
        .spawn((
            Sprite {
                color: Color::srgb(0.8, 0.3, 0.3),
                custom_size: Some(Vec2::new(80.0, 80.0)),
                ..default()
            },
            Transform::from_xyz(400.0, 0.0, 0.0),
            Planet::new("Planet Beta", Vec2::new(400.0, 0.0)),
            PlanetBeta,
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
        Roaming,
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

fn landed_dialog_visibility(
    mut dialog_query: Query<&mut Visibility, With<LandedPlanetDialog>>,
    explorer_roaming: Query<&Explorer, With<Roaming>>,
) {
    if explorer_roaming.is_empty() {
        for mut visibility in &mut dialog_query {
            *visibility = Visibility::Visible;
        }
    } else {
        for mut visibility in &mut dialog_query {
            *visibility = Visibility::Hidden;
        }
    }
}

fn yes_button_system(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<YesButton>)>,
    explorer: Single<Entity, With<Roaming>>,
    mut explorer_query: Single<&mut Transform, With<Explorer>>,
    planet_alpha_entity: Single<Entity, With<PlanetAlpha>>,
    planet_beta_entity: Single<Entity, With<PlanetBeta>>,
    planet_alpha: Single<&Transform, (With<PlanetAlpha>, Without<Explorer>)>,
    planet_beta: Single<&Transform, (With<PlanetBeta>, Without<Explorer>)>,
    mut dialog_query: Query<&mut Visibility, With<PlanetDialog>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            let explorer_entity = *explorer;

            // Determine target planet
            let target_planet = if explorer_query.translation.x < 0.0 {
                *planet_alpha_entity
            } else {
                *planet_beta_entity
            };
            commands.entity(explorer_entity).remove::<Roaming>();
            commands.entity(explorer_entity).insert(Landed {
                planet: target_planet,
            });
            handle_button_press(
                &mut explorer_query,
                &mut dialog_query,
                planet_alpha.translation.x,
                planet_beta.translation.x,
            );
            info!("Yes button pressed");
        }
    }
}
fn no_button_system(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<NoButton>)>,
    mut explorer_query: Single<&mut Transform, With<Explorer>>,
    planet_alpha: Single<&Transform, (With<PlanetAlpha>, Without<Explorer>)>,
    planet_beta: Single<&Transform, (With<PlanetBeta>, Without<Explorer>)>,
    mut dialog_query: Query<&mut Visibility, With<PlanetDialog>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            handle_button_press(
                &mut explorer_query,
                &mut dialog_query,
                planet_alpha.translation.x + 70.0,
                planet_beta.translation.x - 70.0,
            );
            info!("No button pressed");
        }
    }
}

fn handle_button_press(
    explorer_transform: &mut Transform,
    dialog_query: &mut Query<&mut Visibility, With<PlanetDialog>>,
    left_pos: f32,
    right_pos: f32,
) {
    explorer_transform.translation.x = if explorer_transform.translation.x < 0.0 {
        left_pos
    } else {
        right_pos
    };

    for mut visibility in dialog_query {
        *visibility = Visibility::Hidden;
    }
}

fn take_off_button_system(
    mut commands: Commands,
    explorer: Single<Entity, With<Landed>>,
    mut explorer_query: Single<&mut Transform, With<Explorer>>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<TakeOffPlanetButton>)>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            commands.entity(*explorer).remove::<Landed>();
            commands.entity(*explorer).insert(Roaming);
            explorer_query.translation.x = if explorer_query.translation.x < 0.0 {
                explorer_query.translation.x + 70.0
            } else {
                explorer_query.translation.x - 70.0
            };
            info!("No button pressed");
        }
    }
}
