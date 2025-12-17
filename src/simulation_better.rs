use crate::EventSpawnTimer;
use crate::GameState;
use crate::planet::*;
use bevy::prelude::*;

pub fn simulation_better_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Playing), setup)
        .init_resource::<EventSpawnTimer>()
        .add_systems(
            Update,
            (
                crate::galaxy_event::event_spawner_system,
                crate::galaxy_event::event_handler_system,
                crate::galaxy_event::cleanup_events_system,
                crate::galaxy_event::event_visual_system,
            )
                .run_if(in_state(GameState::Playing)),
        );
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let planet1 = commands
        .spawn(planet(
            0,
            "Alpha",
            Vec3::new(400.0, 0.0, 0.0),
            asset_server.load("sprites/Ice.png"),
        ))
        .id();

    commands.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: percent(5.0),
            left: px(20),
            height: percent(90.0),
            width: percent(20.0),
            top: percent(5.0),
            ..default()
        },
        children![
            (planet_state(
                &asset_server,
                "Alpha",
                planet1,
                PlanetCell {
                    num_cell: 5,
                    charged_cell: 0,
                },
                PlanetRocket(false),
            ))
        ],
    ));

    commands.insert_resource(EventSpawnTimer(Timer::from_seconds(
        5.0,
        TimerMode::Repeating,
    )));
}
