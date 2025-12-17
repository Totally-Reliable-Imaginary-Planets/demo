use crate::EventSpawnTimer;
use crate::GameState;
use crate::orchestrator::Orchestrator;
use crate::planet::*;
use bevy::prelude::*;
use common_game::protocols::messages::*;
use crossbeam_channel::*;

pub fn simulation_better_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Playing), setup)
        .init_resource::<EventSpawnTimer>()
        .add_systems(
            Update,
            (
                update_planet_cell,
                update_planet_rocket,
                crate::galaxy_event::event_spawner_system,
                crate::galaxy_event::event_handler_system,
                crate::galaxy_event::cleanup_events_system,
                crate::galaxy_event::event_visual_system,
            )
                .run_if(in_state(GameState::Playing)),
        );
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut orchestrator = Orchestrator::new();

    let (orch_tx_p1, orch_rx_p1) = unbounded();
    let (planet_tx_p1, planet_rx_p1) = unbounded();
    let (_expl_tx_p1, expl_rx_p1) = unbounded();
    orchestrator.add_op_tx(0, orch_tx_p1);
    orchestrator.add_po_rx(0, planet_rx_p1);
    let mut p1 =
        trip::trip(0, orch_rx_p1, planet_tx_p1, expl_rx_p1).expect("Error creating planet1");
    let planet1 = commands
        .spawn(planet(
            0,
            "Alpha",
            Vec3::new(400.0, 0.0, 0.0),
            asset_server.load("sprites/Ice.png"),
        ))
        .id();

    let (orch_tx_p2, orch_rx_p2) = unbounded();
    let (planet_tx_p2, planet_rx_p2) = unbounded();
    let (_expl_tx_p2, expl_rx_p2) = unbounded();
    orchestrator.add_op_tx(1, orch_tx_p2);
    orchestrator.add_po_rx(1, planet_rx_p2);
    let mut p2 =
        trip::trip(1, orch_rx_p2, planet_tx_p2, expl_rx_p2).expect("Error creating planet1");
    let planet2 = commands
        .spawn(planet(
            0,
            "Beta",
            Vec3::new(0.0, 0.0, 0.0),
            asset_server.load("sprites/Terran.png"),
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
            )),
            (planet_state(
                &asset_server,
                "Beta",
                planet2,
                PlanetCell {
                    num_cell: 5,
                    charged_cell: 0,
                },
                PlanetRocket(false),
            )),
        ],
    ));

    commands.insert_resource(EventSpawnTimer(Timer::from_seconds(
        5.0,
        TimerMode::Repeating,
    )));

    let p1_handle = std::thread::spawn(move || {
        let _ = p1.run();
    });
    let p2_handle = std::thread::spawn(move || {
        let _ = p2.run();
    });
    orchestrator.add_planet_handle(0, p1_handle);
    orchestrator.add_planet_handle(1, p2_handle);

    for i in 0..2 {
        orchestrator.send_to_planet_id(i, OrchestratorToPlanet::StartPlanetAI);
        match orchestrator
            .recv_from_planet_id(i)
            .expect("No message received")
        {
            PlanetToOrchestrator::StartPlanetAIResult { planet_id } => {
                info!("Planet {planet_id} started");
            }
            _other => panic!("Failed to start planet"),
        }
    }

    commands.insert_resource(orchestrator);
}

fn update_planet_cell(mut query: Query<(&mut Text, &PlanetCell), Changed<PlanetCell>>) {
    for (mut text, cell) in query.iter_mut() {
        text.0 = cell_string(cell);
    }
}
fn update_planet_rocket(mut query: Query<(&mut Text, &PlanetRocket), Changed<PlanetRocket>>) {
    for (mut text, rocket) in query.iter_mut() {
        if rocket.0 {
            text.0 = "󱎯".to_string();
        } else {
            text.0 = String::new();
        }
    }
}
