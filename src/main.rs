use bevy::prelude::*;

mod explorer;
mod galaxy_event;
mod planet;
mod resources;

use crate::explorer::Explorer;
use crate::planet::Planet;
use crate::resources::PlanetEntities;
use crate::resources::EventSpawnTimer;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                explorer::movement::explorer_movement_system_wasd,
                galaxy_event::event_spawner_system,
                galaxy_event::event_handler_system,
                galaxy_event::cleanup_events_system,
                galaxy_event::event_visual_system,
            ),
        )
        .run();
}

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
        Transform::from_xyz(-300.0, 0.0, 1.0),
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
}
