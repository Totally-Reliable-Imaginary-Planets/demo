use bevy::prelude::*;
use common_game::protocols::messages::*;
use crossbeam_channel::*;

use crate::explorer::Explorer;
use crate::explorer::Landed;
use crate::explorer::Roaming;
use crate::planet::Planet;
use crate::resources::EventSpawnTimer;
use crate::resources::PlanetEntities;

use crate::GameState;
use crate::LandedPlanetDialog;
use crate::LogText;
use crate::NoButton;
use crate::PlanetDialog;
use crate::TakeOffPlanetButton;
use crate::YesButton;

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
            )
                .run_if(in_state(GameState::Playing)),
        );
}

#[derive(Component)]
struct PlanetAlpha;
#[derive(Component)]
struct PlanetBeta;

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
    pub expl_tx: Sender<ExplorerToPlanet>,
    pub expl_rx: Receiver<ExplorerToPlanet>,
}

impl Orchestrator {
    pub fn new() -> Self {
        let (orch_tx_p1, orch_rx_p1) = unbounded();
        let (planet_tx_p1, planet_rx_p1) = unbounded();
        let (orch_tx_p2, orch_rx_p2) = unbounded();
        let (planet_tx_p2, planet_rx_p2) = unbounded();
        let (expl_tx, expl_rx) = unbounded();

        Self {
            orch_tx_p1,
            orch_rx_p1,
            orch_tx_p2,
            orch_rx_p2,
            planet_tx_p1,
            planet_rx_p1,
            planet_tx_p2,
            planet_rx_p2,
            expl_tx,
            expl_rx,
        }
    }

    // Send command to planets
    pub fn send_to_planet_id(
        &self,
        msg: PlanetToOrchestrator,
        id: u32,
    ) -> Result<(), SendError<PlanetToOrchestrator>> {
        if id == 0 {
            return self.planet_tx_p1.send(msg);
        }
        self.planet_tx_p2.send(msg)
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

    // Send explorer data
    pub fn send_explorer_data(
        &self,
        msg: ExplorerToPlanet,
    ) -> Result<(), SendError<ExplorerToPlanet>> {
        self.expl_tx.send(msg)
    }
}

fn setup(mut commands: Commands, mut dialog_query: Query<&mut Visibility, With<LogText>>) {
    let orchestrator = Orchestrator::new();

    // Planets
    let mut p1 = trip::trip(
        0,
        orchestrator.orch_rx_p1.clone(),
        orchestrator.planet_tx_p1.clone(),
        orchestrator.expl_rx.clone(),
    )
    .expect("Error createing planet1");
    let planet1 = commands
        .spawn((
            Sprite {
                color: Color::srgb(0.3, 0.5, 0.8),
                custom_size: Some(Vec2::new(80.0, 80.0)),
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
        orchestrator.expl_rx.clone(),
    )
    .expect("Error createing planet2");
    let planet2 = commands
        .spawn((
            Sprite {
                color: Color::srgb(0.8, 0.3, 0.3),
                custom_size: Some(Vec2::new(80.0, 80.0)),
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
            color: Color::srgb(0.9, 0.9, 0.1),
            custom_size: Some(Vec2::new(30.0, 30.0)),
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
        p1.run();
    });
    std::thread::spawn(move || {
        p2.run();
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
            println!("Planet {planet_id} started")
        }
        _other => panic!("Failed to start planet"),
    }
    match orchestrator
        .planet_rx_p2
        .recv_timeout(std::time::Duration::from_millis(100))
        .expect("No message received")
    {
        PlanetToOrchestrator::StartPlanetAIResult { planet_id } => {
            println!("Planet {planet_id} started")
        }
        _other => panic!("Failed to start planet"),
    }

    commands.insert_resource(orchestrator);

    for mut visibility in &mut dialog_query {
        *visibility = Visibility::Visible;
    }
}

fn check_entities_and_end_game(
    mut commands: Commands,
    planet: Query<&Planet>,
    mut next_state: ResMut<NextState<GameState>>,
    query: Query<Entity, With<Explorer>>,
    mut log_query: Query<&mut Text, With<LogText>>,
    mut dialog_query: Query<&mut Visibility, With<LogText>>,
) {
    if planet.is_empty() {
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
    planet_alpha_entity: Single<Entity, With<PlanetAlpha>>,
    planet_beta_entity: Single<Entity, With<PlanetBeta>>,
    planet_alpha: Single<&Transform, (With<PlanetAlpha>, Without<Explorer>)>,
    planet_beta: Single<&Transform, (With<PlanetBeta>, Without<Explorer>)>,
    mut dialog_query: Query<&mut Visibility, With<PlanetDialog>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            let explorer_entity = *explorer;

            // Determine target planet
            let target_planet = if explorer_query.translation.x < 0.0 {
                *planet_alpha_entity
            } else {
                *planet_beta_entity
            };
            commands.entity(explorer_entity).remove::<Roaming>();
            commands.entity(explorer_entity).insert(Landed {
                planet: target_planet,
            });
            handle_button_press(
                &mut explorer_query,
                &mut dialog_query,
                planet_alpha.translation.x,
                planet_beta.translation.x,
            );
            info!("Yes button pressed");
        }
    }
}
fn no_button_system(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<NoButton>)>,
    mut explorer_query: Single<&mut Transform, With<Explorer>>,
    planet_alpha: Single<&Transform, (With<PlanetAlpha>, Without<Explorer>)>,
    planet_beta: Single<&Transform, (With<PlanetBeta>, Without<Explorer>)>,
    mut dialog_query: Query<&mut Visibility, With<PlanetDialog>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            handle_button_press(
                &mut explorer_query,
                &mut dialog_query,
                planet_alpha.translation.x + 70.0,
                planet_beta.translation.x - 70.0,
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

fn take_off_button_system(
    mut commands: Commands,
    explorer: Single<Entity, With<Landed>>,
    mut explorer_query: Single<&mut Transform, With<Explorer>>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<TakeOffPlanetButton>)>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            commands.entity(*explorer).remove::<Landed>();
            commands.entity(*explorer).insert(Roaming);
            explorer_query.translation.x = if explorer_query.translation.x < 0.0 {
                explorer_query.translation.x + 70.0
            } else {
                explorer_query.translation.x - 70.0
            };
            info!("No button pressed");
        }
    }
}
