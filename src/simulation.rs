use bevy::prelude::*;
use common_game::components::resource::BasicResource;
use common_game::components::resource::BasicResourceType;
use common_game::protocols::messages::*;
use crossbeam_channel::*;

use crate::explorer::Explorer;
use crate::explorer::Landed;
use crate::explorer::Roaming;
use crate::planet::Planet;
use crate::resources::EventSpawnTimer;
use crate::resources::PlanetEntities;

use crate::AvailableEnergyCellButton;
use crate::ExtractResourceButton;
use crate::GameState;
use crate::LandedPlanetDialog;
use crate::LogText;
use crate::NoButton;
use crate::PlanetAlphaCell;
use crate::PlanetAlphaRocket;
use crate::PlanetAlphaState;
use crate::PlanetBetaCell;
use crate::PlanetBetaRocket;
use crate::PlanetBetaState;
use crate::PlanetDialog;
use crate::SupportedResourceButton;
use crate::TakeOffPlanetButton;
use crate::YesButton;

#[derive(Resource)]
pub struct PlanetAlphaStateRes(pub usize, pub usize, pub bool);
#[derive(Resource)]
pub struct PlanetBetaStateRes(pub usize, pub usize, pub bool);

pub fn simulation_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Playing), setup)
        .add_systems(
            Update,
            (
                check_entities_and_end_game,
                landed_dialog_visibility,
                crate::explorer::movement::explorer_movement_system_wasd,
                crate::explorer::movement::check_explorer_reach,
                crate::galaxy_event::event_spawner_system,
                crate::galaxy_event::event_handler_system,
                crate::galaxy_event::cleanup_events_system,
                crate::galaxy_event::event_visual_system,
                yes_button_system,
                no_button_system,
                take_off_button_system,
                supported_resource_button_system,
                available_energy_cell_button_system,
                generate_supported_resource_button_system,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            update_planet_beta_ui.run_if(resource_changed::<PlanetBetaStateRes>),
        )
        .add_systems(
            Update,
            update_planet_alpha_ui.run_if(resource_changed::<PlanetAlphaStateRes>),
        );
}

// Update system for Planet Alpha
fn update_planet_alpha_ui(
    planet_state: Res<PlanetAlphaStateRes>,
    mut cell_query: Query<&mut Text, With<PlanetAlphaCell>>,
    mut rocket_query: Query<&mut Text, (With<PlanetAlphaRocket>, Without<PlanetAlphaCell>)>,
) {
    if planet_state.is_changed() {
        // Update charged cell display
        if let Ok(mut text) = cell_query.single_mut() {
            text.0 = format!("{}/{}", planet_state.1, planet_state.0)
        }

        // Update rocket count
        if let Ok(mut text) = rocket_query.single_mut() {
            text.0 = planet_state.2.to_string();
        }
    }
}

// Update system for Planet Beta
fn update_planet_beta_ui(
    planet_state: Res<PlanetBetaStateRes>,
    mut cell_query: Query<&mut Text, With<PlanetBetaCell>>,
    mut rocket_query: Query<&mut Text, (With<PlanetBetaRocket>, Without<PlanetBetaCell>)>,
) {
    if planet_state.is_changed() {
        // Update charged cell display
        if let Ok(mut text) = cell_query.single_mut() {
            text.0 = format!("{}/{}", planet_state.1, planet_state.0)
        }

        // Update rocket count
        if let Ok(mut text) = rocket_query.single_mut() {
            text.0 = planet_state.2.to_string();
        }
    }
}

#[derive(Component)]
struct PlanetAlpha;
#[derive(Component)]
struct PlanetBeta;

#[derive(Resource)]
pub struct ExplorerHandler {
    pub planet_tx: Sender<PlanetToExplorer>,
    pub planet_rx: Receiver<PlanetToExplorer>,
    pub expl_tx_p1: Sender<ExplorerToPlanet>,
    pub expl_rx_p1: Receiver<ExplorerToPlanet>,
    pub expl_tx_p2: Sender<ExplorerToPlanet>,
    pub expl_rx_p2: Receiver<ExplorerToPlanet>,
    pub id: u32,
}

impl ExplorerHandler {
    pub fn new() -> Self {
        let (planet_tx, planet_rx) = unbounded();
        let (expl_tx_p1, expl_rx_p1) = unbounded();
        let (expl_tx_p2, expl_rx_p2) = unbounded();
        Self {
            planet_tx,
            planet_rx,
            expl_tx_p1,
            expl_rx_p1,
            expl_tx_p2,
            expl_rx_p2,
            id: 0,
        }
    }
}

#[derive(Resource)]
pub struct Orchestrator {
    pub orch_tx_p1: Sender<OrchestratorToPlanet>,
    pub orch_rx_p1: Receiver<OrchestratorToPlanet>,
    pub orch_tx_p2: Sender<OrchestratorToPlanet>,
    pub orch_rx_p2: Receiver<OrchestratorToPlanet>,
    pub planet_tx_p1: Sender<PlanetToOrchestrator>,
    pub planet_rx_p1: Receiver<PlanetToOrchestrator>,
    pub planet_tx_p2: Sender<PlanetToOrchestrator>,
    pub planet_rx_p2: Receiver<PlanetToOrchestrator>,
}

impl Orchestrator {
    pub fn new() -> Self {
        let (orch_tx_p1, orch_rx_p1) = unbounded();
        let (planet_tx_p1, planet_rx_p1) = unbounded();
        let (orch_tx_p2, orch_rx_p2) = unbounded();
        let (planet_tx_p2, planet_rx_p2) = unbounded();

        Self {
            orch_tx_p1,
            orch_rx_p1,
            orch_tx_p2,
            orch_rx_p2,
            planet_tx_p1,
            planet_rx_p1,
            planet_tx_p2,
            planet_rx_p2,
        }
    }

    // Broadcast orchestrator command
    pub fn broadcast(
        &self,
        msg: OrchestratorToPlanet,
        id: u32,
    ) -> Result<(), SendError<OrchestratorToPlanet>> {
        if id == 0 {
            return self.orch_tx_p1.send(msg);
        }
        self.orch_tx_p2.send(msg)
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut dialog_query: Query<
        &mut Visibility,
        Or<(With<LogText>, With<PlanetAlphaState>, With<PlanetBetaState>)>,
    >,
) {
    let orchestrator = Orchestrator::new();
    let explorer_handl = ExplorerHandler::new();

    // Planets
    let mut p1 = trip::trip(
        0,
        orchestrator.orch_rx_p1.clone(),
        orchestrator.planet_tx_p1.clone(),
        explorer_handl.expl_rx_p1.clone(),
    )
    .expect("Error createing planet1");
    let planet1 = commands
        .spawn((
            Sprite {
                image: asset_server.load("sprites/Terran.png"),
                custom_size: Some(Vec2::new(100.0, 100.0)),
                ..default()
            },
            Transform::from_xyz(-400.0, 0.0, 0.0),
            Planet::new("Alpha", Vec2::new(-400.0, 0.0)),
            PlanetAlpha,
        ))
        .id();

    let mut p2 = trip::trip(
        1,
        orchestrator.orch_rx_p2.clone(),
        orchestrator.planet_tx_p2.clone(),
        explorer_handl.expl_rx_p2.clone(),
    )
    .expect("Error createing planet2");
    let planet2 = commands
        .spawn((
            Sprite {
                image: asset_server.load("sprites/Ice.png"),
                custom_size: Some(Vec2::new(100.0, 100.0)),
                ..default()
            },
            Transform::from_xyz(400.0, 0.0, 0.0),
            Planet::new("Beta", Vec2::new(400.0, 0.0)),
            PlanetBeta,
        ))
        .id();

    // Explorer
    commands.spawn((
        Sprite {
            image: asset_server.load("sprites/explorer.png"),
            custom_size: Some(Vec2::new(50.0, 50.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
        Explorer::new(Some(planet2), 150.0),
        Roaming,
    ));

    // Resources
    commands.insert_resource(EventSpawnTimer(Timer::from_seconds(
        5.0,
        TimerMode::Repeating,
    )));
    commands.insert_resource(PlanetEntities {
        planets: vec![planet1, planet2],
    });

    std::thread::spawn(move || {
        let _ = p1.run();
    });
    std::thread::spawn(move || {
        let _ = p2.run();
    });

    orchestrator
        .orch_tx_p1
        .send(OrchestratorToPlanet::StartPlanetAI)
        .expect("Failed to send start messages");
    orchestrator
        .orch_tx_p2
        .send(OrchestratorToPlanet::StartPlanetAI)
        .expect("Failed to send start messages");
    match orchestrator
        .planet_rx_p1
        .recv_timeout(std::time::Duration::from_millis(100))
        .expect("No message received")
    {
        PlanetToOrchestrator::StartPlanetAIResult { planet_id } => {
            info!("Planet {planet_id} started")
        }
        _other => panic!("Failed to start planet"),
    }
    match orchestrator
        .planet_rx_p2
        .recv_timeout(std::time::Duration::from_millis(100))
        .expect("No message received")
    {
        PlanetToOrchestrator::StartPlanetAIResult { planet_id } => {
            info!("Planet {planet_id} started")
        }
        _other => panic!("Failed to start planet"),
    }

    orchestrator
        .orch_tx_p1
        .send(OrchestratorToPlanet::InternalStateRequest)
        .expect("Failed to send start messages");
    orchestrator
        .orch_tx_p2
        .send(OrchestratorToPlanet::InternalStateRequest)
        .expect("Failed to send start messages");
    match orchestrator
        .planet_rx_p1
        .recv_timeout(std::time::Duration::from_millis(100))
        .expect("No message received")
    {
        PlanetToOrchestrator::InternalStateResponse { planet_state, .. } => {
            commands.insert_resource(PlanetAlphaStateRes(
                planet_state.energy_cells.len(),
                planet_state.charged_cells_count,
                planet_state.has_rocket,
            ));
        }
        _other => panic!("Failed to start planet"),
    }
    match orchestrator
        .planet_rx_p2
        .recv_timeout(std::time::Duration::from_millis(100))
        .expect("No message received")
    {
        PlanetToOrchestrator::InternalStateResponse { planet_state, .. } => {
            commands.insert_resource(PlanetBetaStateRes(
                planet_state.energy_cells.len(),
                planet_state.charged_cells_count,
                planet_state.has_rocket,
            ));
        }
        _other => panic!("Failed to start planet"),
    }

    commands.insert_resource(orchestrator);
    commands.insert_resource(explorer_handl);

    for mut visibility in &mut dialog_query {
        *visibility = Visibility::Visible;
    }
}

fn check_entities_and_end_game(
    mut commands: Commands,
    planet: Query<&Planet>,
    explorer: Query<&Planet>,
    mut next_state: ResMut<NextState<GameState>>,
    query: Query<Entity, Or<(With<Explorer>, With<Planet>)>>,
    mut log_query: Query<&mut Text, With<LogText>>,
    mut dialog_query: Query<
        &mut Visibility,
        Or<(With<LogText>, With<PlanetAlphaState>, With<PlanetBetaState>)>,
    >,
) {
    if planet.is_empty() || explorer.is_empty() {
        for mut visibility in &mut dialog_query {
            *visibility = Visibility::Hidden;
        }
        for entity in &query {
            commands.entity(entity).despawn();
        }
        // No player entity found → end game
        next_state.set(GameState::Settings);
        // Update UI text instead of printing
        if let Ok(mut text) = log_query.single_mut() {
            text.0 = String::new();
        }
    }
}

fn landed_dialog_visibility(
    mut dialog_query: Query<&mut Visibility, With<LandedPlanetDialog>>,
    explorer_roaming: Query<&Explorer, With<Roaming>>,
) {
    if explorer_roaming.is_empty() {
        for mut visibility in &mut dialog_query {
            *visibility = Visibility::Visible;
        }
    } else {
        for mut visibility in &mut dialog_query {
            *visibility = Visibility::Hidden;
        }
    }
}

fn yes_button_system(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<YesButton>)>,
    explorer: Single<Entity, With<Roaming>>,
    mut explorer_query: Single<&mut Transform, With<Explorer>>,
    planet_alpha_entity: Query<Entity, With<PlanetAlpha>>,
    planet_beta_entity: Query<Entity, With<PlanetBeta>>,
    planet_alpha: Query<&Transform, (With<PlanetAlpha>, Without<Explorer>)>,
    planet_beta: Query<&Transform, (With<PlanetBeta>, Without<Explorer>)>,
    mut dialog_query: Query<&mut Visibility, With<PlanetDialog>>,
    mut log_query: Query<&mut Text, With<LogText>>,
    orch: Res<Orchestrator>,
    expl: Res<ExplorerHandler>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            let explorer_entity = *explorer;

            // Determine target planet
            let target_planet = if explorer_query.translation.x < 0.0 {
                let _ = orch.broadcast(
                    OrchestratorToPlanet::IncomingExplorerRequest {
                        explorer_id: expl.id,
                        new_mpsc_sender: expl.planet_tx.clone(),
                    },
                    0,
                );
                let res = orch
                    .planet_rx_p1
                    .recv_timeout(std::time::Duration::from_millis(100));

                match res {
                    Ok(msg) => match msg {
                        PlanetToOrchestrator::IncomingExplorerResponse {
                            planet_id,
                            res: Ok(()),
                        } => {
                            info!("Explorer successfully landed on planet {planet_id}");
                        }
                        PlanetToOrchestrator::OutgoingExplorerResponse {
                            planet_id,
                            res: Err(e),
                        } => {
                            warn!(
                                "An error: {e} occurred while the explorer was landing on planet {planet_id}"
                            );
                        }
                        _other => warn!("Wrong message received"),
                    },
                    Err(_) => warn!("Connection timed out"),
                }
                if let Ok(mut text) = log_query.single_mut() {
                    text.0 = format!("Explorer Landed on planet Alpha\n{}", text.0);
                }
                planet_alpha_entity.single().unwrap()
            } else {
                let _ = orch.broadcast(
                    OrchestratorToPlanet::IncomingExplorerRequest {
                        explorer_id: expl.id,
                        new_mpsc_sender: expl.planet_tx.clone(),
                    },
                    1,
                );
                let res = orch
                    .planet_rx_p2
                    .recv_timeout(std::time::Duration::from_millis(100));

                match res {
                    Ok(msg) => match msg {
                        PlanetToOrchestrator::IncomingExplorerResponse {
                            planet_id,
                            res: Ok(()),
                        } => {
                            info!("Explorer successfully landed on planet {planet_id}");
                        }
                        PlanetToOrchestrator::OutgoingExplorerResponse {
                            planet_id,
                            res: Err(e),
                        } => {
                            warn!(
                                "An error: {e} occurred while the explorer was landing on planet {planet_id}"
                            );
                        }
                        _other => warn!("Wrong message received"),
                    },
                    Err(_) => warn!("Connection timed out"),
                }
                if let Ok(mut text) = log_query.single_mut() {
                    text.0 = format!("Explorer Landed on planet Beta\n{}", text.0);
                }
                planet_beta_entity.single().unwrap()
            };
            commands.entity(explorer_entity).remove::<Roaming>();
            commands.entity(explorer_entity).insert(Landed {
                planet: target_planet,
            });
            let planet_alpha_x = match planet_alpha.single() {
                Ok(t) => t.translation.x,
                Err(_) => -300.0,
            };
            let planet_beta_x = match planet_beta.single() {
                Ok(t) => t.translation.x,
                Err(_) => 300.0,
            };
            handle_button_press(
                &mut explorer_query,
                &mut dialog_query,
                planet_alpha_x,
                planet_beta_x,
            );
            info!("Yes button pressed");
        }
    }
}
fn no_button_system(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<NoButton>)>,
    mut explorer_query: Single<&mut Transform, With<Explorer>>,
    planet_alpha: Query<&Transform, (With<PlanetAlpha>, Without<Explorer>)>,
    planet_beta: Query<&Transform, (With<PlanetBeta>, Without<Explorer>)>,
    mut dialog_query: Query<&mut Visibility, With<PlanetDialog>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            let planet_alpha_x = match planet_alpha.single() {
                Ok(t) => t.translation.x,
                Err(_) => -300.0,
            };
            let planet_beta_x = match planet_beta.single() {
                Ok(t) => t.translation.x,
                Err(_) => 300.0,
            };
            handle_button_press(
                &mut explorer_query,
                &mut dialog_query,
                planet_alpha_x + 70.0,
                planet_beta_x - 70.0,
            );
            info!("No button pressed");
        }
    }
}

fn handle_button_press(
    explorer_transform: &mut Transform,
    dialog_query: &mut Query<&mut Visibility, With<PlanetDialog>>,
    left_pos: f32,
    right_pos: f32,
) {
    explorer_transform.translation.x = if explorer_transform.translation.x < 0.0 {
        left_pos
    } else {
        right_pos
    };

    for mut visibility in dialog_query {
        *visibility = Visibility::Hidden;
    }
}

fn available_energy_cell_button_system(
    explorer_query: Single<&Transform, With<Explorer>>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<AvailableEnergyCellButton>)>,
    mut log_query: Query<&mut Text, With<LogText>>,
    expl: Res<ExplorerHandler>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            if explorer_query.translation.x < 0.0 {
                let _ = expl
                    .expl_tx_p1
                    .send(ExplorerToPlanet::AvailableEnergyCellRequest {
                        explorer_id: expl.id,
                    });
                let res = expl
                    .planet_rx
                    .recv_timeout(std::time::Duration::from_millis(100));

                let energy_cell: u32 = match res {
                    Ok(msg) => match msg {
                        PlanetToExplorer::AvailableEnergyCellResponse { available_cells } => {
                            info!("This planet now has {available_cells} charged energy cell");
                            available_cells
                        }
                        _other => {
                            warn!("Wrong message received");
                            0
                        }
                    },
                    Err(_) => {
                        warn!("Connection timed out");
                        0
                    }
                };
                if let Ok(mut text) = log_query.single_mut() {
                    text.0 = format!(
                        "\nPlanet Alpha has {energy_cell} charged energy cell\n{}",
                        text.0
                    );
                }
            } else {
                let _ = expl
                    .expl_tx_p2
                    .send(ExplorerToPlanet::AvailableEnergyCellRequest {
                        explorer_id: expl.id,
                    });
                let res = expl
                    .planet_rx
                    .recv_timeout(std::time::Duration::from_millis(100));

                let energy_cell: u32 = match res {
                    Ok(msg) => match msg {
                        PlanetToExplorer::AvailableEnergyCellResponse { available_cells } => {
                            info!("This planet now has {available_cells} charged energy cell");
                            available_cells
                        }
                        _other => {
                            warn!("Wrong message received");
                            0
                        }
                    },
                    Err(_) => {
                        warn!("Connection timed out");
                        0
                    }
                };
                if let Ok(mut text) = log_query.single_mut() {
                    text.0 = format!(
                        "\nPlanet Beta has {energy_cell} charged energy cell\n{}",
                        text.0
                    );
                }
            }
            info!("No button pressed");
        }
    }
}

fn generate_supported_resource_button_system(
    explorer_query: Single<&Transform, With<Explorer>>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ExtractResourceButton>)>,
    mut log_query: Query<&mut Text, With<LogText>>,
    expl: Res<ExplorerHandler>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            if explorer_query.translation.x < 0.0 {
                let _ = expl
                    .expl_tx_p1
                    .send(ExplorerToPlanet::SupportedResourceRequest {
                        explorer_id: expl.id,
                    });
                let res = expl
                    .planet_rx
                    .recv_timeout(std::time::Duration::from_millis(100));

                let resource: Option<BasicResourceType> = match res {
                    Ok(msg) => match msg {
                        PlanetToExplorer::SupportedResourceResponse { resource_list } => {
                            info!("From this planet you can generate: {:?}", resource_list);
                            Some(*resource_list.iter().next().unwrap())
                        }
                        _other => {
                            warn!("Wrong message received");
                            None
                        }
                    },
                    Err(_) => {
                        warn!("Connection timed out");
                        None
                    }
                };
                let _ = expl
                    .expl_tx_p1
                    .send(ExplorerToPlanet::GenerateResourceRequest {
                        explorer_id: expl.id,
                        resource: resource.unwrap(),
                    });
                let res = expl
                    .planet_rx
                    .recv_timeout(std::time::Duration::from_millis(100));

                let gen_resource: Option<BasicResource> = match res {
                    Ok(msg) => match msg {
                        PlanetToExplorer::GenerateResourceResponse { resource } => {
                            info!("Generated: {:?}", resource);
                            resource
                        }
                        _other => {
                            warn!("Wrong message received");
                            None
                        }
                    },
                    Err(_) => {
                        warn!("Connection timed out");
                        None
                    }
                };
                if let Ok(mut text) = log_query.single_mut() {
                    text.0 = format!(
                        "\nPlanet Alpha has generated: {:?}\n{}",
                        gen_resource, text.0
                    );
                }
            } else {
                let _ = expl
                    .expl_tx_p2
                    .send(ExplorerToPlanet::SupportedResourceRequest {
                        explorer_id: expl.id,
                    });
                let res = expl
                    .planet_rx
                    .recv_timeout(std::time::Duration::from_millis(100));

                let resource: Option<BasicResourceType> = match res {
                    Ok(msg) => match msg {
                        PlanetToExplorer::SupportedResourceResponse { resource_list } => {
                            info!("From this planet you can generate: {:?}", resource_list);
                            Some(*resource_list.iter().next().unwrap())
                        }
                        _other => {
                            warn!("Wrong message received");
                            None
                        }
                    },
                    Err(_) => {
                        warn!("Connection timed out");
                        None
                    }
                };
                let _ = expl
                    .expl_tx_p2
                    .send(ExplorerToPlanet::GenerateResourceRequest {
                        explorer_id: expl.id,
                        resource: resource.unwrap(),
                    });
                let res = expl
                    .planet_rx
                    .recv_timeout(std::time::Duration::from_millis(100));

                let gen_resource: Option<BasicResource> = match res {
                    Ok(msg) => match msg {
                        PlanetToExplorer::GenerateResourceResponse { resource } => {
                            info!("Generated: {:?}", resource);
                            resource
                        }
                        _other => {
                            warn!("Wrong message received");
                            None
                        }
                    },
                    Err(_) => {
                        warn!("Connection timed out");
                        None
                    }
                };
                if let Ok(mut text) = log_query.single_mut() {
                    text.0 = format!(
                        "\nPlanet Beta has generated: {:?}\n{}",
                        gen_resource, text.0
                    );
                }
            }
            info!("No button pressed");
        }
    }
}

fn supported_resource_button_system(
    explorer_query: Single<&Transform, With<Explorer>>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<SupportedResourceButton>)>,
    mut log_query: Query<&mut Text, With<LogText>>,
    expl: Res<ExplorerHandler>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            if explorer_query.translation.x < 0.0 {
                let _ = expl
                    .expl_tx_p1
                    .send(ExplorerToPlanet::SupportedResourceRequest {
                        explorer_id: expl.id,
                    });
                let res = expl
                    .planet_rx
                    .recv_timeout(std::time::Duration::from_millis(100));

                let resource: Option<BasicResourceType> = match res {
                    Ok(msg) => match msg {
                        PlanetToExplorer::SupportedResourceResponse { resource_list } => {
                            info!("From this planet you can generate: {:?}", resource_list);
                            Some(*resource_list.iter().next().unwrap())
                        }
                        _other => {
                            warn!("Wrong message received");
                            None
                        }
                    },
                    Err(_) => {
                        warn!("Connection timed out");
                        None
                    }
                };
                if let Ok(mut text) = log_query.single_mut() {
                    text.0 = format!("\nPlanet Alpha can generate: {:?}\n{}", resource, text.0);
                }
            } else {
                let _ = expl
                    .expl_tx_p2
                    .send(ExplorerToPlanet::SupportedResourceRequest {
                        explorer_id: expl.id,
                    });
                let res = expl
                    .planet_rx
                    .recv_timeout(std::time::Duration::from_millis(100));

                let resource: Option<BasicResourceType> = match res {
                    Ok(msg) => match msg {
                        PlanetToExplorer::SupportedResourceResponse { resource_list } => {
                            info!("From this planet you can generate: {:?}", resource_list);
                            Some(*resource_list.iter().next().unwrap())
                        }
                        _other => {
                            warn!("Wrong message received");
                            None
                        }
                    },
                    Err(_) => {
                        warn!("Connection timed out");
                        None
                    }
                };
                if let Ok(mut text) = log_query.single_mut() {
                    text.0 = format!("\nPlanet Beta can generate: {:?}\n{}", resource, text.0);
                }
            }
            info!("No button pressed");
        }
    }
}

fn take_off_button_system(
    mut commands: Commands,
    explorer: Single<Entity, With<Landed>>,
    mut explorer_query: Single<&mut Transform, With<Explorer>>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<TakeOffPlanetButton>)>,
    mut log_query: Query<&mut Text, With<LogText>>,
    orch: Res<Orchestrator>,
    expl: Res<ExplorerHandler>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            commands.entity(*explorer).remove::<Landed>();
            commands.entity(*explorer).insert(Roaming);
            explorer_query.translation.x = if explorer_query.translation.x < 0.0 {
                let _ = orch.broadcast(
                    OrchestratorToPlanet::OutgoingExplorerRequest {
                        explorer_id: expl.id,
                    },
                    0,
                );
                let res = orch
                    .planet_rx_p1
                    .recv_timeout(std::time::Duration::from_millis(100));

                match res {
                    Ok(msg) => match msg {
                        PlanetToOrchestrator::OutgoingExplorerResponse {
                            planet_id,
                            res: Ok(()),
                        } => {
                            info!("Explorer successfully take off from planet {planet_id}");
                        }
                        PlanetToOrchestrator::OutgoingExplorerResponse {
                            planet_id,
                            res: Err(e),
                        } => {
                            warn!(
                                "An error: {e} occurred while the explorer take off from planet {planet_id}"
                            );
                        }
                        _other => warn!("Wrong message received"),
                    },
                    Err(_) => warn!("Connection timed out"),
                }
                if let Ok(mut text) = log_query.single_mut() {
                    text.0 = format!("\nExplorer take off from planet Alpha\n{}", text.0);
                }
                explorer_query.translation.x + 70.0
            } else {
                let _ = orch.broadcast(
                    OrchestratorToPlanet::OutgoingExplorerRequest {
                        explorer_id: expl.id,
                    },
                    1,
                );
                let res = orch
                    .planet_rx_p2
                    .recv_timeout(std::time::Duration::from_millis(100));
                match res {
                    Ok(msg) => match msg {
                        PlanetToOrchestrator::OutgoingExplorerResponse {
                            planet_id,
                            res: Ok(()),
                        } => {
                            info!("Explorer successfully take off from planet {planet_id}");
                        }
                        PlanetToOrchestrator::OutgoingExplorerResponse {
                            planet_id,
                            res: Err(e),
                        } => {
                            warn!(
                                "An error: {e} occurred while the explorer take off from planet {planet_id}"
                            );
                        }
                        _other => warn!("Wrong message received"),
                    },
                    Err(_) => warn!("Connection timed out"),
                }
                if let Ok(mut text) = log_query.single_mut() {
                    text.0 = format!("\nExplorer take off from planet Beta\n{}", text.0);
                }
                explorer_query.translation.x - 70.0
            };
            info!("No button pressed");
        }
    }
}
