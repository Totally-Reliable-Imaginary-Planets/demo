use crate::orchestrator::Orchestrator;
use bevy::prelude::*;
use common_game::components::asteroid::Asteroid;
use common_game::components::sunray::Sunray;
use common_game::protocols::messages::OrchestratorToPlanet;
use common_game::protocols::messages::PlanetToOrchestrator;
use rand::Rng;

use crate::EventSpawnTimer;
use crate::LogText;
use crate::PlanetEntities;
use crate::planet::*;

#[derive(Component)]
pub enum GalaxyEvent {
    Sunray,
    Asteroid,
}

#[derive(Component)]
pub struct EventTarget {
    planet: Entity,
    duration: Timer,
}

#[derive(Component)]
pub struct EventVisual;

fn handle_spawn_sunray_event(commands: &mut Commands, target: Entity, name: &str) -> String {
    commands.spawn((
        GalaxyEvent::Sunray,
        EventTarget {
            planet: target,
            duration: Timer::from_seconds(3.0, TimerMode::Once),
        },
    ));
    format!(" Sunray approaching planet {name}!")
}

pub fn event_spawner_system(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<EventSpawnTimer>,
    existing_events: Query<&EventTarget>,
    planet_query: Query<(Entity, &Name, &PlanetId), With<Planet>>,
    ui_query: Query<(Entity, &PlanetUi)>,
    children_query: Query<&Children, With<PlanetUi>>,
    mut cell_query: Query<&mut PlanetCell>,
    mut rocket_query: Query<&mut PlanetRocket>,
    //mut log_query: Query<&mut Text, With<LogText>>,
    mut orch: ResMut<Orchestrator>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }
    if !existing_events.is_empty() {
        return;
    }

    let mut rng = rand::rng();

    // Choose random planet
    let planet_idx = rng.random_range(0..planet_query.count());

    let Some((target, name, id)) = planet_query.iter().nth(planet_idx) else {
        warn!("no planet finded with id {planet_idx}");
        return;
    };
    let Some((entity, _)) = ui_query.iter().find(|&(_, ui)| ui.0 == target) else {
        warn!("no ui finded for such planet");
        return;
    };
    let Ok(children) = children_query.get(entity) else {
        warn!("no children founded for such ui");
        return;
    };
    let log_message = match rng.random_range(0..3) {
        0 => handle_spawn_sunray_event(&mut commands, target, name),
        1 => {
            orch.send_to_planet_id(id.0, OrchestratorToPlanet::Asteroid(Asteroid::default()));
            match orch.recv_from_planet_id(id.0) {
                Ok(msg) => match msg {
                    PlanetToOrchestrator::AsteroidAck { rocket: None, .. } => {
                        commands.spawn((
                            GalaxyEvent::Asteroid,
                            EventTarget {
                                planet: target,
                                duration: Timer::from_seconds(3.0, TimerMode::Once),
                            },
                        ));
                        orch.send_to_planet_id(id.0, OrchestratorToPlanet::KillPlanet);
                        match orch.recv_from_planet_id(id.0) {
                            Ok(msg) => match msg {
                                PlanetToOrchestrator::KillPlanetResult { planet_id } => {
                                    info!("planet {planet_id} kiled successfully")
                                }
                                _other => warn!("Wrong message received"),
                            },
                            Err(e) => {
                                warn!(
                                    "An error occurred while waiting or request timed out, Err: {e}"
                                );
                            }
                        }
                        orch.join_planet_id(id.0);
                        format!(" Asteroid approaching planet {name}!")
                    }
                    PlanetToOrchestrator::AsteroidAck {
                        rocket: Some(_), ..
                    } => {
                        orch.send_to_planet_id(id.0, OrchestratorToPlanet::InternalStateRequest);

                        match orch.recv_from_planet_id(id.0) {
                            Ok(msg) => match msg {
                                PlanetToOrchestrator::InternalStateResponse {
                                    planet_state,
                                    ..
                                } => {
                                    for child in children.iter() {
                                        if let Ok(mut cell) = cell_query.get_mut(child) {
                                            cell.num_cell = planet_state.energy_cells.len();
                                            cell.charged_cell = planet_state.charged_cells_count;
                                        }
                                        if let Ok(mut rocket) = rocket_query.get_mut(child) {
                                            rocket.0 = planet_state.has_rocket;
                                        }
                                    }
                                }
                                _other => warn!("Wrong message received"),
                            },
                            Err(e) => {
                                warn!(
                                    "An error occurred while waiting or request timed out, Err: {e}"
                                );
                            }
                        }
                        format!(" Asteroid approaching planet {name} Was destroyed by a rocket 󱎯",)
                    }
                    _other => "Wrong message received".to_string(),
                },
                Err(e) => format!("Error {e}"),
            }
        }
        _ => "󰒲 Nothing happening this cycle.".to_string(),
    };

    info!(log_message);
    // Update UI text instead of printing
    /*if let Ok(mut text) = log_query.single_mut() {
        text.0 = format!("\n{}\n{}", log_message, text.0);
    }*/
}
pub fn event_visual_spawn(
    event: On<Add, GalaxyEvent>,
    mut commands: Commands,
    event_query: Query<(&GalaxyEvent, &EventTarget, Entity), Without<EventVisual>>,
    planet_query: Query<&Transform, With<Planet>>,
) {
    // Create visuals for new events
    let Ok((event_type, target, event_entity)) = event_query.get(event.entity) else {
        return;
    };
    let Ok(transform) = planet_query.get(target.planet) else {
        return;
    };
    let (color, size) = match event_type {
        GalaxyEvent::Sunray => (Color::srgb(1.0, 1.0, 0.0), Vec2::new(40.0, 40.0)),
        GalaxyEvent::Asteroid => (Color::srgb(0.5, 0.5, 0.5), Vec2::new(35.0, 35.0)),
    };

    commands.entity(event_entity).insert((
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_xyz(
            transform.translation.x,
            transform.translation.y + 100.0,
            2.0,
        ),
        EventVisual,
    ));
}

pub fn event_visual_move(
    mut commands: Commands,
    planet_query: Query<&Transform, (With<Planet>, Without<EventVisual>)>,
    mut existing_visuals: Query<(&mut Transform, &mut Sprite), With<EventVisual>>,
) {
    // Animate existing visuals (simple bobbing effect)
    for (mut transform, mut sprite) in existing_visuals.iter_mut() {
        transform.translation.y -= 20.0 * 0.016;

        let new_alpha = (sprite.color.alpha() - 0.01).max(0.3);
        sprite.color.set_alpha(new_alpha);
    }
}

// ===== Event Handler System =====

pub fn event_handler_system(
    mut commands: Commands,
    time: Res<Time>,
    mut event_query: Single<(&GalaxyEvent, &mut EventTarget)>,
    planet_query: Query<(&Name, &PlanetId), With<Planet>>,
    ui_query: Query<(Entity, &PlanetUi)>,
    children_query: Query<&Children, With<PlanetUi>>,
    mut cell_query: Query<&mut PlanetCell>,
    mut rocket_query: Query<&mut PlanetRocket>,
    //mut log_query: Query<&mut Text, With<LogText>>,
    orch: Res<Orchestrator>,
) {
    let (event, mut target) = event_query.into_inner();
    target.duration.tick(time.delta());
    if !target.duration.just_finished() {
        return;
    }
    let Ok((name, id)) = planet_query.get(target.planet) else {
        return;
    };
    let Some((entity, _)) = ui_query.iter().find(|&(_, ui)| ui.0 == target.planet) else {
        return;
    };
    let Ok(children) = children_query.get(entity) else {
        return;
    };
    let log_message = match event {
        GalaxyEvent::Sunray => {
            let res = {
                orch.send_to_planet_id(id.0, OrchestratorToPlanet::Sunray(Sunray::default()));
                let res = orch.recv_from_planet_id(id.0);

                orch.send_to_planet_id(id.0, OrchestratorToPlanet::InternalStateRequest);

                match orch.recv_from_planet_id(id.0) {
                    Ok(msg) => match msg {
                        PlanetToOrchestrator::InternalStateResponse { planet_state, .. } => {
                            for child in children.iter() {
                                if let Ok(mut cell) = cell_query.get_mut(child) {
                                    cell.num_cell = planet_state.energy_cells.len();
                                    cell.charged_cell = planet_state.charged_cells_count;
                                }
                                if let Ok(mut rocket) = rocket_query.get_mut(child) {
                                    rocket.0 = planet_state.has_rocket;
                                }
                            }
                        }
                        _other => warn!("Wrong message received"),
                    },
                    Err(e) => {
                        warn!("An error occurred while waiting or request timed out, Err: {e}");
                    }
                }
                res
            };

            match res {
                Ok(msg) => match msg {
                    PlanetToOrchestrator::SunrayAck { planet_id } => {
                        info!("Sunray received by {planet_id}");
                    }
                    _other => warn!("Wrong message received"),
                },
                Err(e) => warn!("Error {e}"),
            }
            format!("󰂄 Sunray hit {name}! Energy increased.")
        }
        GalaxyEvent::Asteroid => {
            commands.entity(target.planet).despawn();
            commands.entity(entity).despawn();
            format!("󰈸 Asteroid hit {name}! Planet destroyed.")
        }
    };

    // Update UI text instead of printing
    info!(log_message);
    /*if let Ok(mut text) = log_query.single_mut() {
        text.0 = format!("\n{}\n{}", log_message, text.0);
    }*/
}

// ===== Cleanup Events System =====

pub fn cleanup_events_system(mut commands: Commands, event_query: Single<(Entity, &EventTarget)>) {
    let (entity, target) = event_query.into_inner();
    if !target.duration.is_finished() {
        return;
    }
    commands.entity(entity).despawn();
}
