use crate::EventSpawnTimer;
use crate::GameState;
use crate::galaxy_event::*;
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
                crate::galaxy_event::event_spawner_system,
                crate::galaxy_event::event_handler_system,
                crate::galaxy_event::cleanup_events_system,
                crate::galaxy_event::event_visual_spawn,
                crate::galaxy_event::event_visual_move,
                check_entities_and_end_game,
                update_planet_cell,
                update_planet_rocket,
            )
                .run_if(in_state(GameState::Playing)),
        );
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut orchestrator = Orchestrator::new();
    let mut id = 0;

    let (orch_tx, orch_rx) = unbounded();
    let (planet_tx, planet_rx) = unbounded();
    let (_expl_tx, expl_rx) = unbounded();
    orchestrator.add_op_tx(id, orch_tx);
    orchestrator.add_po_rx(id, planet_rx);
    let mut p1 = trip::trip(id, orch_rx, planet_tx, expl_rx).expect("Error creating planet1");
    let planet1 = commands
        .spawn(planet(
            id,
            "Alpha",
            Vec3::new(400.0, 0.0, 0.0),
            asset_server.load("sprites/Ice.png"),
        ))
        .id();

    let p1_handle = std::thread::spawn(move || {
        let _ = p1.run();
    });
    orchestrator.add_planet_handle(id, p1_handle);

    id += 1;
    let (orch_tx, orch_rx) = unbounded();
    let (planet_tx, planet_rx) = unbounded();
    let (_expl_tx, expl_rx) = unbounded();
    orchestrator.add_op_tx(id, orch_tx);
    orchestrator.add_po_rx(id, planet_rx);
    let mut p2 = trip::trip(id, orch_rx, planet_tx, expl_rx).expect("Error creating planet1");
    let planet2 = commands
        .spawn(planet(
            id,
            "Beta",
            Vec3::new(0.0, 0.0, 0.0),
            asset_server.load("sprites/Terran.png"),
        ))
        .id();
    let p2_handle = std::thread::spawn(move || {
        let _ = p2.run();
    });
    orchestrator.add_planet_handle(id, p2_handle);

    id += 1;
    let (orch_tx, orch_rx) = unbounded();
    let (planet_tx, planet_rx) = unbounded();
    let (_expl_tx, expl_rx) = unbounded();
    orchestrator.add_op_tx(id, orch_tx);
    orchestrator.add_po_rx(id, planet_rx);
    let mut p3 = trip::trip(id, orch_rx, planet_tx, expl_rx).expect("Error creating planet1");
    let planet3 = commands
        .spawn(planet(
            id,
            "Gamma",
            Vec3::new(0.0, 200.0, 0.0),
            asset_server.load("sprites/Terran.png"),
        ))
        .id();
    let p3_handle = std::thread::spawn(move || {
        let _ = p3.run();
    });
    orchestrator.add_planet_handle(id, p3_handle);

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
            (planet_state(
                &asset_server,
                "Gamma",
                planet3,
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

    for i in 0..=id {
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

fn check_entities_and_end_game(
    planet: Query<&Planet>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    if !planet.is_empty() {
        return;
    }
    // No player entity found → end game
    next_state.set(current_state.next());
}
