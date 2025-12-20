use crate::orchestrator::Orchestrator;
use bevy::prelude::*;
use common_game::components::asteroid::Asteroid;
use common_game::components::sunray::Sunray;
use common_game::protocols::orchestrator_planet::OrchestratorToPlanet;
use rand::Rng;

use crate::EventSpawnTimer;
use crate::GameState;
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

pub fn event_spawner_system(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<EventSpawnTimer>,
    planet_query: Query<(Entity, &Name, &PlanetId), With<Planet>>,
    //mut log_query: Query<&mut Text, With<LogText>>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let mut rng = rand::rng();

    // Choose random planet
    let planet_idx = rng.random_range(0..planet_query.count());

    let Some((target, name, id)) = planet_query.iter().nth(planet_idx) else {
        warn!("no planet finded with id {planet_idx}");
        return;
    };
    let log_message = match rng.random_range(0..3) {
        0 => {
    commands.spawn((
        DespawnOnExit(GameState::Playing),
        GalaxyEvent::Sunray,
        EventTarget {
            planet: target,
            duration: Timer::from_seconds(3.0, TimerMode::Once),
        },
    ));
    format!(" Sunray approaching planet {name}!")
}
        1 => {
            commands.spawn((
                DespawnOnExit(GameState::Playing),
                GalaxyEvent::Asteroid,
                EventTarget {
                    planet: target,
                    duration: Timer::from_seconds(3.0, TimerMode::Once),
                },
            ));
            format!(" Asteroid approaching planet {name}!")
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
    time: Res<Time>,
    mut event_query: Query<(&GalaxyEvent, &mut EventTarget)>,
    planet_query: Query<&PlanetId, With<Planet>>,
    //mut log_query: Query<&mut Text, With<LogText>>,
    orch: Res<Orchestrator>,
) {
    for (event, mut target) in event_query.iter_mut() {
        target.duration.tick(time.delta());
        if !target.duration.just_finished() {
            continue;
        }
        let Ok(id) = planet_query.get(target.planet) else {
            continue;
        };
        match event {
            GalaxyEvent::Sunray => {
                orch.send_to_planet_id(id.0, OrchestratorToPlanet::Sunray(Sunray::default()));
            }
            GalaxyEvent::Asteroid => {
                orch.send_to_planet_id(id.0, OrchestratorToPlanet::Asteroid(Asteroid::default()));
            }
        };

        // Update UI text instead of printing
        /*if let Ok(mut text) = log_query.single_mut() {
            text.0 = format!("\n{}\n{}", log_message, text.0);
        }*/
    }
}

// ===== Cleanup Events System =====

pub fn cleanup_events_system(mut commands: Commands, event_query: Query<(Entity, &EventTarget)>) {
    for (entity, target) in event_query.iter() {
        if !target.duration.is_finished() {
            continue;
        }
        commands.entity(entity).despawn();
    }
}
