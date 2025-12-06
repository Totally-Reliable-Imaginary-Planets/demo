use bevy::prelude::*;
use rand::Rng;

mod planet;

use crate::planet::Planet;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                explorer_movement_system,
                event_spawner_system,
                event_handler_system,
                cleanup_events_system,
                event_visual_system,
            ),
        )
        .run();
}

// ===== Components =====
#[derive(Component)]
struct Explorer {
    target_planet: Option<Entity>,
    travel_speed: f32,
}

#[derive(Component)]
enum GalaxyEvent {
    Sunray,
    Asteroid,
}

#[derive(Component)]
struct EventTarget {
    planet: Entity,
    duration: Timer,
}

#[derive(Component)]
struct EventVisual;

// ===== Resources =====

#[derive(Resource)]
struct EventSpawnTimer(Timer);

#[derive(Resource)]
struct PlanetEntities {
    planets: Vec<Entity>,
}

// ===== Setup System =====

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn((
        Camera2d::default(),
        Camera::default(),
        Transform::from_xyz(0.0, 0.0, 1000.0),
    ));

    // Planet 1
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

    // Planet 2
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
        Transform::from_xyz(-300.0, 100.0, 1.0),
        Explorer {
            target_planet: Some(planet2),
            travel_speed: 150.0,
        },
    ));

    // Resources
    commands.insert_resource(EventSpawnTimer(Timer::from_seconds(
        5.0,
        TimerMode::Repeating,
    )));
    commands.insert_resource(PlanetEntities {
        planets: vec![planet1, planet2],
    });
}

// ===== Explorer Movement System =====

fn explorer_movement_system(
    time: Res<Time>,
    mut explorer_query: Query<(&mut Transform, &mut Explorer)>,
    planet_query: Query<&Planet>,
    planet_entities: Res<PlanetEntities>,
) {
    for (mut transform, mut explorer) in explorer_query.iter_mut() {
        if let Some(target_entity) = explorer.target_planet {
            if let Ok(target_planet) = planet_query.get(target_entity) {
                let current_pos = Vec2::new(transform.translation.x, transform.translation.y);
                let direction = target_planet.position() - current_pos;
                let distance = direction.length();

                if distance > 5.0 {
                    // Still traveling
                    let movement =
                        direction.normalize() * explorer.travel_speed * time.delta_secs();
                    transform.translation.x += movement.x;
                    transform.translation.y += movement.y;
                } else {
                    // Arrived, switch to other planet
                    let other_planet = planet_entities
                        .planets
                        .iter()
                        .find(|&&p| p != target_entity)
                        .copied();
                    explorer.target_planet = other_planet;
                    println!("Explorer arrived at {}!", target_planet.name());
                }
            }
        }
    }
}

// ===== Event Spawner System =====

fn event_spawner_system(
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

// ===== Event Visual System =====

fn event_visual_system(
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

fn event_handler_system(
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

fn cleanup_events_system(mut commands: Commands, event_query: Query<(Entity, &EventTarget)>) {
    for (entity, target) in event_query.iter() {
        if target.duration.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
