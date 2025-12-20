use crate::EventSpawnTimer;
use crate::GameState;
use crate::galaxy_event::*;
use crate::orchestrator::Orchestrator;
use crate::planet::*;
use bevy::prelude::*;
use common_game::protocols::orchestrator_planet::*;
use common_game::protocols::planet_explorer::*;
use crossbeam_channel::*;

pub fn simulation_better_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Playing), setup)
        .init_resource::<EventSpawnTimer>()
        .add_systems(
            Update,
            (
                crate::galaxy_event::event_spawner_system,
                listen_to_planets,
                crate::galaxy_event::event_handler_system,
                crate::galaxy_event::cleanup_events_system,
                crate::galaxy_event::event_visual_move,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            PostUpdate,
            (
                check_entities_and_end_game,
                update_planet_cell,
                update_planet_rocket,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_observer(event_visual_spawn);
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
        1.0,
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

fn listen_to_planets(
    mut commands: Commands,
    orch: Res<Orchestrator>,
    planet_query: Query<(&PlanetId, Entity), With<Planet>>,
    ui_query: Query<(Entity, &PlanetUi)>,
    children_query: Query<&Children, With<PlanetUi>>,
    mut cell_query: Query<&mut PlanetCell>,
    mut rocket_query: Query<&mut PlanetRocket>,
) {
    for rx in orch.planet_rxs() {
        match rx.try_recv() {
            Ok(msg) => match msg {
                PlanetToOrchestrator::SunrayAck { planet_id } => {
                    orch.send_to_planet_id(planet_id, OrchestratorToPlanet::InternalStateRequest);
                    info!("Sunray received by {planet_id}");
                }
                PlanetToOrchestrator::AsteroidAck { planet_id, rocket } => match rocket {
                    Some(_) => {
                        info!(" Asteroid approaching planet {planet_id} Was destroyed by a rocket 󱎯",);
                        orch.send_to_planet_id(
                            planet_id,
                            OrchestratorToPlanet::InternalStateRequest,
                        );
                    }
                    None => {
                        let Some((id, planet_entity)) =
                            planet_query.iter().find(|&(id, _)| id.0 == planet_id)
                        else {
                            return;
                        };
                        let Some((entity, _)) =
                            ui_query.iter().find(|&(_, ui)| ui.0 == planet_entity)
                        else {
                            return;
                        };
                        commands.entity(planet_entity).despawn();
                        commands.entity(entity).despawn();
                        orch.send_to_planet_id(planet_id, OrchestratorToPlanet::KillPlanet);
                    }
                },
                PlanetToOrchestrator::StartPlanetAIResult { planet_id } => {}
                PlanetToOrchestrator::StopPlanetAIResult { planet_id } => {}
                PlanetToOrchestrator::KillPlanetResult { planet_id } => {
                    //TODO; send an event to join the planet thread
                    //  orch.join_planet_id(planet_id);
                    info!("planet {planet_id} killed successfully");
                }
                PlanetToOrchestrator::InternalStateResponse {
                    planet_id,
                    planet_state,
                } => {
                    let Some((id, planet_entity)) =
                        planet_query.iter().find(|&(id, _)| id.0 == planet_id)
                    else {
                        return;
                    };
                    let Some((entity, _)) = ui_query.iter().find(|&(_, ui)| ui.0 == planet_entity)
                    else {
                        return;
                    };
                    let Ok(children) = children_query.get(entity) else {
                        return;
                    };

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
                PlanetToOrchestrator::IncomingExplorerResponse { planet_id, res, explorer_id } => {}
                PlanetToOrchestrator::OutgoingExplorerResponse { planet_id, res, explorer_id } => {}
                PlanetToOrchestrator::Stopped { planet_id } => {}
            },
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {}
        };
    }
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
