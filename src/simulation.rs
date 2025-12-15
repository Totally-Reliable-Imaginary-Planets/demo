use bevy::prelude::*;
use common_game::components::resource::BasicResource;
use common_game::components::resource::BasicResourceType;
use common_game::protocols::messages::{
    ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator,
};
use crossbeam_channel::{Receiver, SendError, Sender, unbounded};
use std::collections::HashMap;
use std::thread::JoinHandle;

use crate::explorer::Explorer;
use crate::explorer::Landed;
use crate::explorer::Roaming;
use crate::planet::Planet;
use crate::resources::EventSpawnTimer;
use crate::resources::PlanetEntities;
use crate::theme;

use crate::AvailableEnergyCellButton;
use crate::ExtractResourceButton;
use crate::GameState;
use crate::LandedPlanetDialog;
use crate::LogScreen;
use crate::LogText;
use crate::NoButton;
use crate::PlanetAlphaState;
use crate::PlanetBetaState;
use crate::PlanetCell;
use crate::PlanetDialog;
use crate::PlanetRocket;
use crate::SupportedResourceButton;
use crate::TakeOffPlanetButton;
use crate::YesButton;

#[derive(Resource)]
pub struct PlanetAlphaStateRes(pub usize, pub usize, pub bool);
#[derive(Resource)]
pub struct PlanetBetaStateRes(pub usize, pub usize, pub bool);
#[derive(Resource, Default)]
pub struct PlanetStates {
    inner: HashMap<PlanetId, PlanetState>,
}
pub struct PlanetState(pub usize, pub usize, pub bool);
#[derive(Hash, PartialEq, Eq, Clone)]
enum PlanetId {
    Alpha,
    Beta,
}

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
                update_planet_beta_ui.run_if(resource_changed::<PlanetBetaStateRes>),
                update_planet_alpha_ui.run_if(resource_changed::<PlanetAlphaStateRes>),
            )
                .run_if(in_state(GameState::Playing)),
        );
}

fn cell_string(charged_cell: usize, empty_cell: usize) -> String {
    let mut cells = String::new();
    cells.push_str(&"󰁹 ".repeat(charged_cell));
    cells.push_str(&"󰁺 ".repeat(empty_cell));
    cells
}

// Update system for Planet Alpha
fn update_planet_alpha_ui(
    planet_state: Res<PlanetAlphaStateRes>,
    planet_query: Query<&Children, With<PlanetAlphaState>>,
    mut cell_query: Query<&mut Text, With<PlanetCell>>,
    mut rocket_query: Query<&mut Text, (With<PlanetRocket>, Without<PlanetCell>)>,
) {
    if planet_state.is_changed() {
        for children in &planet_query {
            for &child in children {
                if let Ok(mut text) = cell_query.get_mut(child) {
                    text.0 = cell_string(planet_state.1, planet_state.0 - planet_state.1);
                }
                if let Ok(mut text) = rocket_query.get_mut(child) {
                    if planet_state.2 {
                        text.0 = "󱎯".to_string();
                    } else {
                        text.0 = String::new();
                    }
                }
            }
        }
    }
}

// Update system for Planet Beta
fn update_planet_beta_ui(
    planet_state: Res<PlanetBetaStateRes>,
    planet_query: Query<&Children, With<PlanetBetaState>>,
    mut cell_query: Query<&mut Text, With<PlanetCell>>,
    mut rocket_query: Query<&mut Text, (With<PlanetRocket>, Without<PlanetCell>)>,
) {
    if planet_state.is_changed() {
        for children in &planet_query {
            for &child in children {
                if let Ok(mut text) = cell_query.get_mut(child) {
                    text.0 = cell_string(planet_state.1, planet_state.0 - planet_state.1);
                }
                if let Ok(mut text) = rocket_query.get_mut(child) {
                    if planet_state.2 {
                        text.0 = "󱎯".to_string();
                    } else {
                        text.0 = String::new();
                    }
                }
            }
        }
    }
}

#[derive(Component)]
struct PlanetAlpha;
#[derive(Component)]
struct PlanetBeta;

#[derive(Resource)]
pub struct ExplorerHandler {
    planet_tx: Sender<PlanetToExplorer>,
    planet_rx: Receiver<PlanetToExplorer>,
    expl_tx: HashMap<u32, Sender<ExplorerToPlanet>>,
    id: u32,
}

impl ExplorerHandler {
    pub fn new() -> Self {
        let (planet_tx, planet_rx) = unbounded();
        Self {
            planet_tx,
            planet_rx,
            expl_tx: HashMap::new(),
            id: 0,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn planet_tx(&self) -> Sender<PlanetToExplorer> {
        self.planet_tx.clone()
    }

    pub fn add_ep_tx(&mut self, id: u32, tx: Sender<ExplorerToPlanet>) {
        self.expl_tx.insert(id, tx);
    }

    pub fn recv_from_planet(
        &self,
    ) -> Result<PlanetToExplorer, crossbeam_channel::RecvTimeoutError> {
        self.planet_rx
            .recv_timeout(std::time::Duration::from_millis(100))
    }

    pub fn send_to_planet_id(
        &self,
        id: u32,
        msg: ExplorerToPlanet,
    ) -> Result<(), SendError<ExplorerToPlanet>> {
        self.expl_tx.get(&id).unwrap().send(msg)
    }
}

#[derive(Resource)]
pub struct Orchestrator {
    orch_tx: HashMap<u32, Sender<OrchestratorToPlanet>>,
    planet_rx: HashMap<u32, Receiver<PlanetToOrchestrator>>,
    planet_handle: HashMap<u32, JoinHandle<()>>,
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            orch_tx: HashMap::new(),
            planet_rx: HashMap::new(),
            planet_handle: HashMap::new(),
        }
    }

    pub fn add_op_tx(&mut self, id: u32, tx: Sender<OrchestratorToPlanet>) {
        self.orch_tx.insert(id, tx);
    }
    pub fn add_po_rx(&mut self, id: u32, rx: Receiver<PlanetToOrchestrator>) {
        self.planet_rx.insert(id, rx);
    }
    pub fn add_planet_handle(&mut self, id: u32, handle: JoinHandle<()>) {
        self.planet_handle.insert(id, handle);
    }

    pub fn join_planet_id(&mut self, id: u32) {
        match self.planet_handle.remove(&id).unwrap().join() {
            Ok(()) => info!("planet {id} joined successfully"),
            Err(e) => error!("and error {:?} occurred while joining the planet {id}", e),
        }
    }

    pub fn send_to_planet_id(&self, id: u32, msg: OrchestratorToPlanet) {
        info!("attempting to send message {:?} to planet {id}", &msg);
        match self.orch_tx.get(&id).unwrap().send(msg) {
            Ok(()) => {
                info!("Sended message to planet {id}")
            }
            Err(e) => warn!(
                "an error {:?} occurred while sending message to planet {id}",
                e
            ),
        }
    }

    pub fn recv_from_planet_id(
        &self,
        id: u32,
    ) -> Result<PlanetToOrchestrator, crossbeam_channel::RecvTimeoutError> {
        self.planet_rx
            .get(&id)
            .unwrap()
            .recv_timeout(std::time::Duration::from_millis(100))
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let (orch_tx_p1, orch_rx_p1) = unbounded();
    let (planet_tx_p1, planet_rx_p1) = unbounded();
    let (expl_tx_p1, expl_rx_p1) = unbounded();
    let mut orchestrator = Orchestrator::new();
    let mut explorer_handl = ExplorerHandler::new();

    // Planets
    orchestrator.add_op_tx(0, orch_tx_p1);
    orchestrator.add_po_rx(0, planet_rx_p1);
    explorer_handl.add_ep_tx(0, expl_tx_p1);
    let mut p1 =
        trip::trip(0, orch_rx_p1, planet_tx_p1, expl_rx_p1).expect("Error creating planet1");
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

    let (orch_tx_p2, orch_rx_p2) = unbounded();
    let (planet_tx_p2, planet_rx_p2) = unbounded();
    let (expl_tx_p2, expl_rx_p2) = unbounded();
    orchestrator.add_op_tx(1, orch_tx_p2);
    orchestrator.add_po_rx(1, planet_rx_p2);
    explorer_handl.add_ep_tx(1, expl_tx_p2);
    let mut p2 =
        trip::trip(1, orch_rx_p2, planet_tx_p2, expl_rx_p2).expect("Error createing planet2");
    let planet2 = commands
        .spawn((
            DespawnOnExit(GameState::Playing),
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
        DespawnOnExit(GameState::Playing),
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

    let p1_handle = std::thread::spawn(move || {
        let _ = p1.run();
    });
    let p2_handle = std::thread::spawn(move || {
        let _ = p2.run();
    });

    orchestrator.add_planet_handle(0, p1_handle);
    orchestrator.add_planet_handle(1, p2_handle);

    orchestrator.send_to_planet_id(0, OrchestratorToPlanet::StartPlanetAI);
    orchestrator.send_to_planet_id(1, OrchestratorToPlanet::StartPlanetAI);
    match orchestrator
        .recv_from_planet_id(0)
        .expect("No message received")
    {
        PlanetToOrchestrator::StartPlanetAIResult { planet_id } => {
            info!("Planet {planet_id} started");
        }
        _other => panic!("Failed to start planet"),
    }
    match orchestrator
        .recv_from_planet_id(1)
        .expect("No message received")
    {
        PlanetToOrchestrator::StartPlanetAIResult { planet_id } => {
            info!("Planet {planet_id} started");
        }
        _other => panic!("Failed to start planet"),
    }

    orchestrator.send_to_planet_id(0, OrchestratorToPlanet::InternalStateRequest);
    orchestrator.send_to_planet_id(1, OrchestratorToPlanet::InternalStateRequest);
    match orchestrator
        .recv_from_planet_id(0)
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
        .recv_from_planet_id(1)
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
    let spacing = 2.0;
    let padding = 12.0;

    // Planet states
    commands.spawn(create_planet_state(
        &asset_server,
        "Planet Alpha",
        Val::Auto,
        Val::Percent(spacing),
        Val::Percent(spacing),
        PlanetAlphaState,
        PlanetCell,
        PlanetRocket,
    ));

    commands.spawn(create_planet_state(
        &asset_server,
        "Planet Beta",
        Val::Percent(spacing),
        Val::Auto,
        Val::Percent(spacing),
        PlanetBetaState,
        PlanetCell,
        PlanetRocket,
    ));

    // Log screen
    commands.spawn((
        DespawnOnExit(GameState::Playing),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(0.0),
            left: Val::Percent(2.5),
            width: Val::Percent(95.0),
            height: Val::Percent(30.0),
            padding: UiRect::all(Val::Px(padding)),
            overflow: Overflow::scroll_y(),
            ..default()
        },
        LogScreen,
        Visibility::Visible,
        theme::background_color(),
        children![(
            Text::new(""),
            theme::basic_font(&asset_server),
            theme::text_color(),
            LogText,
        )],
    ));

    commands.spawn((
        DespawnOnExit(GameState::Playing),
        Node {
            flex_direction: FlexDirection::Column,
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::SpaceBetween,
            bottom: Val::Percent(35.0),
            left: Val::Percent(35.0),
            width: Val::Percent(30.0),
            min_width: Val::Px(400.0),
            height: Val::Percent(60.0),
            ..default()
        },
        LandedPlanetDialog,
        Visibility::Hidden,
        theme::background_color(),
        children![
            (
                Text::new("What would you do on this planet?"),
                theme::title_font(&asset_server),
                theme::text_color(),
            ),
            create_button(
                &asset_server,
                "Supported Resource",
                SupportedResourceButton,
                Val::Percent(90.0),
                Val::Percent(15.0),
                Val::Percent(5.0),
                Val::Px(5.0),
                Color::WHITE
            ),
            create_button(
                &asset_server,
                "Extract Resource",
                ExtractResourceButton,
                Val::Percent(90.0),
                Val::Percent(15.0),
                Val::Percent(5.0),
                Val::Px(5.0),
                Color::WHITE
            ),
            create_button(
                &asset_server,
                "Available Energy Cell",
                AvailableEnergyCellButton,
                Val::Percent(90.0),
                Val::Percent(15.0),
                Val::Percent(5.0),
                Val::Px(5.0),
                Color::WHITE
            ),
            create_button(
                &asset_server,
                "Take off",
                TakeOffPlanetButton,
                Val::Percent(90.0),
                Val::Percent(15.0),
                Val::Percent(5.0),
                Val::Px(5.0),
                Color::WHITE
            ),
        ],
    ));

    // Spawn land_on_planet_dialog UI
    commands.spawn(land_on_planet_dialog(&asset_server));
}

fn create_planet_state(
    asset_server: &Res<AssetServer>,
    planet_name: &str,
    right: Val,
    left: Val,
    top: Val,
    state: impl Component,
    cell: impl Component,
    rocket: impl Component,
) -> impl Bundle {
    let padding = 12.0;
    let width = 10.0;
    let max_width = width + 5.0;
    let height = 15.0;

    (
        DespawnOnExit(GameState::Playing),
        Node {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            top: top,
            right: right,
            left: left,
            padding: UiRect::all(Val::Px(padding)),
            width: Val::Percent(width),
            max_width: Val::Percent(max_width),
            height: Val::Percent(height),
            ..default()
        },
        state,
        Visibility::Visible,
        theme::background_color(),
        children![
            (
                Text::new(planet_name),
                theme::title_font(asset_server),
                theme::text_color(),
            ),
            (
                Text::new("Energy cell:"),
                theme::basic_font(asset_server),
                theme::text_color(),
            ),
            (
                Text::new(""),
                theme::basic_font(asset_server),
                theme::text_color(),
                cell
            ),
            (
                Text::new("Rocket:"),
                theme::basic_font(asset_server),
                theme::text_color(),
            ),
            (
                Text::new(""),
                theme::basic_font(asset_server),
                theme::text_color(),
                rocket,
            )
        ],
    )
}

fn land_on_planet_dialog(asset_server: &Res<AssetServer>) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Percent(30.0),
            left: Val::Percent(30.0),
            width: Val::Percent(40.0),
            height: Val::Percent(40.0),
            ..default()
        },
        Visibility::Hidden,
        theme::background_color(),
        PlanetDialog,
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),

                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            children![
                (
                    Text::new("You have reached a planet do you want to land on it?"),
                    theme::title_font(asset_server),
                    theme::text_color(),
                ),
                (
                    Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    children![
                        create_button(
                            &asset_server,
                            "Yes",
                            YesButton,
                            Val::Percent(40.0),
                            Val::Percent(100.0),
                            Val::Auto,
                            Val::Px(5.0),
                            Color::WHITE
                        ),
                        create_button(
                            &asset_server,
                            "No",
                            NoButton,
                            Val::Percent(40.0),
                            Val::Percent(100.0),
                            Val::Auto,
                            Val::Px(5.0),
                            Color::WHITE
                        ),
                    ],
                )
            ],
        )],
    )
}

fn create_button(
    asset_server: &Res<AssetServer>,
    text: &str,
    button_component: impl Component,
    width: Val,
    height: Val,
    left: Val,
    border_stroke: Val,
    border_color: Color,
) -> impl Bundle {
    (
        Button,
        button_component,
        Node {
            width: width,
            height: height,
            left: left,
            border: UiRect::all(border_stroke),
            // horizontally center child text
            justify_content: JustifyContent::Center,
            // vertically center child text
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor::all(border_color),
        children![(
            Text::new(text),
            theme::title_font(asset_server),
            theme::text_color(),
        )],
    )
}

fn check_entities_and_end_game(
    planet: Query<&Planet>,
    explorer: Query<&Planet>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if planet.is_empty() || explorer.is_empty() {
        // No player entity found → end game
        next_state.set(GameState::Settings);
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
                orch.send_to_planet_id(
                    0,
                    OrchestratorToPlanet::IncomingExplorerRequest {
                        explorer_id: expl.id(),
                        new_mpsc_sender: expl.planet_tx(),
                    },
                );
                let res = orch.recv_from_planet_id(0);

                if let Ok(msg) = res {
                    match msg {
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
                    }
                } else {
                    warn!("Connection timed out");
                }
                if let Ok(mut text) = log_query.single_mut() {
                    text.0 = format!("Explorer Landed on planet Alpha\n{}", text.0);
                }
                planet_alpha_entity.single().unwrap()
            } else {
                orch.send_to_planet_id(
                    1,
                    OrchestratorToPlanet::IncomingExplorerRequest {
                        explorer_id: expl.id(),
                        new_mpsc_sender: expl.planet_tx(),
                    },
                );
                let res = orch.recv_from_planet_id(1);

                if let Ok(msg) = res {
                    match msg {
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
                    }
                } else {
                    warn!("Connection timed out");
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
                let _ = expl.send_to_planet_id(
                    0,
                    ExplorerToPlanet::AvailableEnergyCellRequest {
                        explorer_id: expl.id(),
                    },
                );
                let res = expl.recv_from_planet();

                let energy_cell: u32 = if let Ok(msg) = res {
                    match msg {
                        PlanetToExplorer::AvailableEnergyCellResponse { available_cells } => {
                            info!("This planet now has {available_cells} charged energy cell");
                            available_cells
                        }
                        _other => {
                            warn!("Wrong message received");
                            0
                        }
                    }
                } else {
                    warn!("Connection timed out");
                    0
                };
                if let Ok(mut text) = log_query.single_mut() {
                    text.0 = format!(
                        "\nPlanet Alpha has {energy_cell} charged energy cell\n{}",
                        text.0
                    );
                }
            } else {
                let _ = expl.send_to_planet_id(
                    1,
                    ExplorerToPlanet::AvailableEnergyCellRequest {
                        explorer_id: expl.id(),
                    },
                );
                let res = expl.recv_from_planet();

                let energy_cell: u32 = if let Ok(msg) = res {
                    match msg {
                        PlanetToExplorer::AvailableEnergyCellResponse { available_cells } => {
                            info!("This planet now has {available_cells} charged energy cell");
                            available_cells
                        }
                        _other => {
                            warn!("Wrong message received");
                            0
                        }
                    }
                } else {
                    warn!("Connection timed out");
                    0
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
    orch: Res<Orchestrator>,
    mut planet_alpha_state: ResMut<PlanetAlphaStateRes>,
    mut planet_beta_state: ResMut<PlanetBetaStateRes>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            if explorer_query.translation.x < 0.0 {
                let _ = expl.send_to_planet_id(
                    0,
                    ExplorerToPlanet::SupportedResourceRequest {
                        explorer_id: expl.id(),
                    },
                );
                let res = expl.recv_from_planet();

                let resource: Option<BasicResourceType> = if let Ok(msg) = res {
                    match msg {
                        PlanetToExplorer::SupportedResourceResponse { resource_list } => {
                            info!("From this planet you can generate: {:?}", resource_list);
                            Some(*resource_list.iter().next().unwrap())
                        }
                        _other => {
                            warn!("Wrong message received");
                            None
                        }
                    }
                } else {
                    warn!("Connection timed out");
                    None
                };
                let _ = expl.send_to_planet_id(
                    0,
                    ExplorerToPlanet::GenerateResourceRequest {
                        explorer_id: expl.id(),
                        resource: resource.unwrap(),
                    },
                );
                let res = expl.recv_from_planet();

                let gen_resource: Option<BasicResource> = if let Ok(msg) = res {
                    match msg {
                        PlanetToExplorer::GenerateResourceResponse { resource } => {
                            info!("Generated: {:?}", resource);
                            resource
                        }
                        _other => {
                            warn!("Wrong message received");
                            None
                        }
                    }
                } else {
                    warn!("Connection timed out");
                    None
                };
                if let Ok(mut text) = log_query.single_mut() {
                    text.0 = format!(
                        "\nPlanet Alpha has generated: {:?}\n{}",
                        gen_resource, text.0
                    );
                }
                orch.send_to_planet_id(0, OrchestratorToPlanet::InternalStateRequest);

                match orch.recv_from_planet_id(0) {
                    Ok(msg) => match msg {
                        PlanetToOrchestrator::InternalStateResponse { planet_state, .. } => {
                            planet_alpha_state.1 = planet_state.charged_cells_count;
                            planet_alpha_state.2 = planet_state.has_rocket;
                        }
                        _other => warn!("Wrong message received"),
                    },
                    Err(e) => {
                        warn!("An error occurred while waiting or request timed out, Err: {e}");
                    }
                }
            } else {
                let _ = expl.send_to_planet_id(
                    1,
                    ExplorerToPlanet::SupportedResourceRequest {
                        explorer_id: expl.id(),
                    },
                );
                let res = expl.recv_from_planet();

                let resource: Option<BasicResourceType> = if let Ok(msg) = res {
                    match msg {
                        PlanetToExplorer::SupportedResourceResponse { resource_list } => {
                            info!("From this planet you can generate: {:?}", resource_list);
                            Some(*resource_list.iter().next().unwrap())
                        }
                        _other => {
                            warn!("Wrong message received");
                            None
                        }
                    }
                } else {
                    warn!("Connection timed out");
                    None
                };
                let _ = expl.send_to_planet_id(
                    1,
                    ExplorerToPlanet::GenerateResourceRequest {
                        explorer_id: expl.id(),
                        resource: resource.unwrap(),
                    },
                );
                let res = expl.recv_from_planet();

                let gen_resource: Option<BasicResource> = if let Ok(msg) = res {
                    match msg {
                        PlanetToExplorer::GenerateResourceResponse { resource } => {
                            info!("Generated: {:?}", resource);
                            resource
                        }
                        _other => {
                            warn!("Wrong message received");
                            None
                        }
                    }
                } else {
                    warn!("Connection timed out");
                    None
                };
                if let Ok(mut text) = log_query.single_mut() {
                    text.0 = format!(
                        "\nPlanet Beta has generated: {:?}\n{}",
                        gen_resource, text.0
                    );
                }
                orch.send_to_planet_id(1, OrchestratorToPlanet::InternalStateRequest);

                match orch.recv_from_planet_id(1) {
                    Ok(msg) => match msg {
                        PlanetToOrchestrator::InternalStateResponse { planet_state, .. } => {
                            planet_beta_state.1 = planet_state.charged_cells_count;
                            planet_beta_state.2 = planet_state.has_rocket;
                        }
                        _other => warn!("Wrong message received"),
                    },
                    Err(e) => {
                        warn!("An error occurred while waiting or request timed out, Err: {e}");
                    }
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
                let _ = expl.send_to_planet_id(
                    0,
                    ExplorerToPlanet::SupportedResourceRequest {
                        explorer_id: expl.id(),
                    },
                );
                let res = expl.recv_from_planet();

                let resource: Option<BasicResourceType> = if let Ok(msg) = res {
                    match msg {
                        PlanetToExplorer::SupportedResourceResponse { resource_list } => {
                            info!("From this planet you can generate: {:?}", resource_list);
                            Some(*resource_list.iter().next().unwrap())
                        }
                        _other => {
                            warn!("Wrong message received");
                            None
                        }
                    }
                } else {
                    warn!("Connection timed out");
                    None
                };
                if let Ok(mut text) = log_query.single_mut() {
                    text.0 = format!("\nPlanet Alpha can generate: {:?}\n{}", resource, text.0);
                }
            } else {
                let _ = expl.send_to_planet_id(
                    1,
                    ExplorerToPlanet::SupportedResourceRequest {
                        explorer_id: expl.id(),
                    },
                );
                let res = expl.recv_from_planet();

                let resource: Option<BasicResourceType> = if let Ok(msg) = res {
                    match msg {
                        PlanetToExplorer::SupportedResourceResponse { resource_list } => {
                            info!("From this planet you can generate: {:?}", resource_list);
                            Some(*resource_list.iter().next().unwrap())
                        }
                        _other => {
                            warn!("Wrong message received");
                            None
                        }
                    }
                } else {
                    warn!("Connection timed out");
                    None
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
                let _ = orch.send_to_planet_id(
                    0,
                    OrchestratorToPlanet::OutgoingExplorerRequest {
                        explorer_id: expl.id(),
                    },
                );
                let res = orch.recv_from_planet_id(0);

                if let Ok(msg) = res {
                    match msg {
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
                    }
                } else {
                    warn!("Connection timed out");
                }
                if let Ok(mut text) = log_query.single_mut() {
                    text.0 = format!("\nExplorer take off from planet Alpha\n{}", text.0);
                }
                explorer_query.translation.x + 70.0
            } else {
                let _ = orch.send_to_planet_id(
                    1,
                    OrchestratorToPlanet::OutgoingExplorerRequest {
                        explorer_id: expl.id(),
                    },
                );
                let res = orch.recv_from_planet_id(1);
                if let Ok(msg) = res {
                    match msg {
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
                    }
                } else {
                    warn!("Connection timed out");
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
