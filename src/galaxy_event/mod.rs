use bevy::prelude::Alpha;
use bevy::prelude::Color;
use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::Entity;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::Sprite;
use bevy::prelude::Text;
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
use crate::LogText;
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
    planet_query: Query<&Planet>,
    mut log_query: Query<&mut Text, With<LogText>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        // Don't spawn if there's already an active event
        if existing_events.is_empty() {
            let mut rng = rand::rng();

            // Choose random planet
            let planet_idx = rng.random_range(0..planet_entities.planets.len());
            let target_planet = planet_entities.planets[planet_idx];
            if let Ok(planet) = planet_query.get(target_planet) {
                // Choose random event (33% each: Sunray, Asteroid, Nothing)
                let log_message = match rng.random_range(0..3) {
                    0 => {
                        commands.spawn((
                            GalaxyEvent::Sunray,
                            EventTarget {
                                planet: target_planet,
                                duration: Timer::from_seconds(3.0, TimerMode::Once),
                            },
                        ));
                        format!("☀️ Sunray approaching planet {}!", planet.name())
                    }
                    1 => {
                        commands.spawn((
                            GalaxyEvent::Asteroid,
                            EventTarget {
                                planet: target_planet,
                                duration: Timer::from_seconds(3.0, TimerMode::Once),
                            },
                        ));
                        format!("☄️ Asteroid approaching planet {}!", planet.name())
                    }
                    _ => "🌌 Nothing happening this cycle.".to_string(),
                };

                // Update UI text instead of printing
                if let Ok(mut text) = log_query.single_mut() {
                    text.0 = format!("{}\n{}", log_message, text.0);
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
    planet_query: Query<&Planet>,
    mut log_query: Query<&mut Text, With<LogText>>,
) {
    for (event, mut target) in &mut event_query {
        target.duration.tick(time.delta());

        if target.duration.just_finished()
            && let Ok(planet) = planet_query.get(target.planet)
        {
            let log_message = match event {
                GalaxyEvent::Sunray => {
                    format!("✨ Sunray hit {}! Energy increased.", planet.name())
                }
                GalaxyEvent::Asteroid => {
                    //commands.entity(target.planet).despawn();
                    format!("💥 Asteroid hit {}! Damage taken.", planet.name())
                }
            };

            // Update UI text instead of printing
            if let Ok(mut text) = log_query.single_mut() {
                text.0 = format!("{}\n{}", log_message, text.0);
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
