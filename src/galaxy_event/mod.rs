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
/*use crate::simulation::PlanetAlphaStateRes;
use crate::simulation::PlanetBetaStateRes;
use crate::simulation::PlanetState;
use crate::simulation::PlanetStates;
use crate::simulation::PlanetToUpdate;*/

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

pub fn event_spawner_system(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<EventSpawnTimer>,
    existing_events: Query<&EventTarget>,
    planet_query: Query<(Entity, &PlanetName, &PlanetId), With<Planet>>,
    //mut log_query: Query<&mut Text, With<LogText>>,
    //mut orch: ResMut<Orchestrator>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        // Don't spawn if there's already an active event
        if existing_events.is_empty() {
            let mut rng = rand::rng();

            // Choose random planet
            let planet_idx = rng.random_range(0..planet_query.count());
            if let Some((target, name, id)) = planet_query.iter().nth(planet_idx) {
                // Choose random event (33% each: Sunray, Asteroid, Nothing)
                let log_message = match rng.random_range(0..3) {
                    0 => {
                        commands.spawn((
                            GalaxyEvent::Sunray,
                            EventTarget {
                                planet: target,
                                duration: Timer::from_seconds(3.0, TimerMode::Once),
                            },
                        ));
                        format!(" Sunray approaching planet {}!", name.0)
                    }
                    1 => {
                        /*let planet_id = if planet.name() == "Alpha" { 0 } else { 1 };
                        let res = {
                            orch.send_to_planet_id(
                                planet_id,
                                OrchestratorToPlanet::Asteroid(Asteroid::default()),
                            );
                            let res = orch.recv_from_planet_id(planet_id);

                            orch.send_to_planet_id(
                                planet_id,
                                OrchestratorToPlanet::InternalStateRequest,
                            );

                            match orch.recv_from_planet_id(planet_id) {
                                Ok(msg) => match msg {
                                    PlanetToOrchestrator::InternalStateResponse {
                                        planet_state,
                                        ..
                                    } => {
                                        planet_states.insert(
                                            planet_id,
                                            PlanetState {
                                                num_cell: planet_state.energy_cells.len(),
                                                charged_cell: planet_state.charged_cells_count,
                                                has_rocket: planet_state.has_rocket,
                                            },
                                        );
                                        planet_to_update.0 = planet_id;
                                        if planet_id == 0 {
                                            planet_alpha_state.1 = planet_state.charged_cells_count;
                                            planet_alpha_state.2 = planet_state.has_rocket;
                                        } else {
                                            planet_beta_state.1 = planet_state.charged_cells_count;
                                            planet_beta_state.2 = planet_state.has_rocket;
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
                            res
                        };
                        match res {
                            Ok(msg) => match msg {
                                PlanetToOrchestrator::AsteroidAck { rocket: None, .. } => {
                                    commands.spawn((
                                        GalaxyEvent::Asteroid,
                                        EventTarget {
                                            planet: target_planet,
                                            duration: Timer::from_seconds(3.0, TimerMode::Once),
                                        },
                                    ));
                                    orch.send_to_planet_id(
                                        planet_id,
                                        OrchestratorToPlanet::KillPlanet,
                                    );
                                    match orch.recv_from_planet_id(planet_id) {
                                        Ok(msg) => match msg {
                                            PlanetToOrchestrator::KillPlanetResult {
                                                planet_id,
                                            } => info!("planet {planet_id} kiled successfully"),
                                            _other => warn!("Wrong message received"),
                                        },
                                        Err(e) => {
                                            warn!(
                                                "An error occurred while waiting or request timed out, Err: {e}"
                                            );
                                        }
                                    }
                                    orch.join_planet_id(planet_id);
                                    format!(" Asteroid approaching planet {}!", planet.name())
                                }
                                PlanetToOrchestrator::AsteroidAck {
                                    rocket: Some(_), ..
                                } => {
                                    format!(
                                        " Asteroid approaching planet {} Was destroyed by a rocket 󱎯",
                                        planet.name()
                                    )
                                }
                                _other => "Wrong message received".to_string(),
                            },
                            Err(e) => format!("Error {e}"),
                        }*/
                        commands.spawn((
                            GalaxyEvent::Asteroid,
                            EventTarget {
                                planet: target,
                                duration: Timer::from_seconds(3.0, TimerMode::Once),
                            },
                        ));
                        format!(
                            " Asteroid approaching planet {} Was destroyed by a rocket 󱎯",
                            name.0
                        )
                    }
                    _ => "󰒲 Nothing happening this cycle.".to_string(),
                };

                info!(log_message);
                // Update UI text instead of printing
                /*if let Ok(mut text) = log_query.single_mut() {
                    text.0 = format!("\n{}\n{}", log_message, text.0);
                }*/
            }
        }
    }
}

pub fn event_visual_system(
    mut commands: Commands,
    event_query: Query<(&GalaxyEvent, &EventTarget, Entity), Without<EventVisual>>,
    planet_query: Query<&Transform, (With<Planet>, Without<EventVisual>)>,
    mut existing_visuals: Query<(&mut Transform, &mut Sprite), With<EventVisual>>,
) {
    // Create visuals for new events
    for (event, target, event_entity) in event_query.iter() {
        if let Ok(transform) = planet_query.get(target.planet) {
            let (color, size) = match event {
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
    }

    // Animate existing visuals (simple bobbing effect)
    for (mut transform, mut sprite) in &mut existing_visuals {
        transform.translation.y -= 20.0 * 0.016;

        let new_alpha = (sprite.color.alpha() - 0.01).max(0.3);
        sprite.color.set_alpha(new_alpha);
    }
}

// ===== Event Handler System =====

pub fn event_handler_system(
    mut commands: Commands,
    time: Res<Time>,
    mut event_query: Query<(&GalaxyEvent, &mut EventTarget)>,
    planet_query: Query<(&PlanetName, &PlanetId), With<Planet>>,
    //mut log_query: Query<&mut Text, With<LogText>>,
    //orch: Res<Orchestrator>,
) {
    for (event, mut target) in &mut event_query {
        target.duration.tick(time.delta());

        if target.duration.just_finished()
            && let Ok((name, id)) = planet_query.get(target.planet)
        {
            let log_message = match event {
                GalaxyEvent::Sunray => {
                    /*let res = {
                        orch.send_to_planet_id(
                            planet_id,
                            OrchestratorToPlanet::Sunray(Sunray::default()),
                        );
                        let res = orch.recv_from_planet_id(planet_id);

                        orch.send_to_planet_id(
                            planet_id,
                            OrchestratorToPlanet::InternalStateRequest,
                        );

                        match orch.recv_from_planet_id(planet_id) {
                            Ok(msg) => match msg {
                                PlanetToOrchestrator::InternalStateResponse {
                                    planet_state,
                                    ..
                                } => {
                                    planet_states.insert(
                                        planet_id,
                                        PlanetState {
                                            num_cell: planet_state.energy_cells.len(),
                                            charged_cell: planet_state.charged_cells_count,
                                            has_rocket: planet_state.has_rocket,
                                        },
                                    );
                                    planet_to_update.0 = planet_id;
                                    if planet_id == 0 {
                                        planet_alpha_state.1 = planet_state.charged_cells_count;
                                        planet_alpha_state.2 = planet_state.has_rocket;
                                    } else {
                                        planet_beta_state.1 = planet_state.charged_cells_count;
                                        planet_beta_state.2 = planet_state.has_rocket;
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
                    }*/
                    format!("󰂄 Sunray hit {}! Energy increased.", name.0)
                }
                GalaxyEvent::Asteroid => {
                    //commands.entity(target.planet).despawn();
                    format!("󰈸 Asteroid hit {}! Planet destroyed.", name.0)
                }
            };

            // Update UI text instead of printing
            info!(log_message);
            /*if let Ok(mut text) = log_query.single_mut() {
                text.0 = format!("\n{}\n{}", log_message, text.0);
            }*/
        }
    }
}

// ===== Cleanup Events System =====

pub fn cleanup_events_system(mut commands: Commands, event_query: Query<(Entity, &EventTarget)>) {
    for (entity, target) in event_query.iter() {
        if target.duration.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
