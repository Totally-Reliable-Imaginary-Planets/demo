use bevy::prelude::Alpha;
use bevy::prelude::Color;
use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::Entity;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::Sprite;
use bevy::prelude::Time;
use bevy::prelude::Timer;
use bevy::prelude::TimerMode;
use bevy::prelude::Transform;
use bevy::prelude::Vec2;
use bevy::prelude::With;
use bevy::prelude::Without;
use bevy::prelude::default;
use rand::Rng;

use crate::EventSpawnTimer;
use crate::Planet;
use crate::PlanetEntities;

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
    planet_entities: Res<PlanetEntities>,
    existing_events: Query<&EventTarget>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        // Don't spawn if there's already an active event
        if existing_events.is_empty() {
            let mut rng = rand::rng();

            // Choose random planet
            let planet_idx = rng.random_range(0..planet_entities.planets.len());
            let target_planet = planet_entities.planets[planet_idx];

            // Choose random event (33% each: Sunray, Asteroid, Nothing)
            let event_type = rng.random_range(0..3);

            match event_type {
                0 => {
                    println!("☀️ Sunray approaching planet!");
                    commands.spawn((
                        GalaxyEvent::Sunray,
                        EventTarget {
                            planet: target_planet,
                            duration: Timer::from_seconds(3.0, TimerMode::Once),
                        },
                    ));
                }
                1 => {
                    println!("☄️ Asteroid approaching planet!");
                    commands.spawn((
                        GalaxyEvent::Asteroid,
                        EventTarget {
                            planet: target_planet,
                            duration: Timer::from_seconds(3.0, TimerMode::Once),
                        },
                    ));
                }
                _ => {
                    println!("🌌 Nothing happening this cycle.");
                }
            }
        }
    }
}

pub fn event_visual_system(
    mut commands: Commands,
    event_query: Query<(&GalaxyEvent, &EventTarget, Entity), Without<EventVisual>>,
    planet_query: Query<&Planet>,
    mut existing_visuals: Query<(&mut Transform, &mut Sprite), With<EventVisual>>,
) {
    // Create visuals for new events
    for (event, target, event_entity) in event_query.iter() {
        if let Ok(planet) = planet_query.get(target.planet) {
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
                Transform::from_xyz(planet.position().x, planet.position().y + 100.0, 2.0),
                EventVisual,
            ));
        }
    }

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
    planet_query: Query<&Planet>,
) {
    for (event, mut target) in event_query.iter_mut() {
        target.duration.tick(time.delta());

        if target.duration.just_finished() {
            if let Ok(planet) = planet_query.get(target.planet) {
                match event {
                    GalaxyEvent::Sunray => {
                        println!("✨ Sunray hit {}! Energy increased.", planet.name());
                    }
                    GalaxyEvent::Asteroid => {
                        println!("💥 Asteroid hit {}! Damage taken.", planet.name());
                    }
                }
            }
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
