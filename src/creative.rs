use super::GameState;
use crate::galaxy_event::*;
use crate::orchestrator::Orchestrator;
use crate::planet::*;
use crate::simulation_better::*;
use crate::theme;
use bevy::prelude::*;
use common_game::protocols::orchestrator_planet::OrchestratorToPlanet;
use common_game::protocols::orchestrator_planet::PlanetToOrchestrator;
use crossbeam_channel::unbounded;

#[derive(Component)]
struct SettingsDialog;

pub fn creative_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Creative), setup)
        .add_systems(
            Update,
            (
                create_event_listen,
                crate::galaxy_event::event_visual_move,
                crate::galaxy_event::event_handler_system,
                listen_to_planets,
                crate::galaxy_event::cleanup_events_system,
            )
                .chain()
                .run_if(in_state(GameState::Creative)),
        )
        .add_systems(
            PostUpdate,
            (
                check_entities_and_end_game,
                update_planet_cell,
                update_planet_rocket,
            )
                .run_if(in_state(GameState::Creative)),
        )
        .add_observer(event_visual_spawn);
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut orchestrator = Orchestrator::new();
    let mut id = 0;

    let (orch_tx, orch_rx) = unbounded();
    let (planet_tx, planet_rx) = unbounded();
    let (_expl_tx, expl_rx) = unbounded();
    orchestrator.add_op_tx(id, orch_tx);
    orchestrator.add_po_rx(id, planet_rx);
    // Planets
    let mut p1 = trip::trip(id, orch_rx.clone(), planet_tx.clone(), expl_rx.clone())
        .expect("Error createing planet1");
    let planet1 = commands
        .spawn(planet(
            id,
            "Alpha",
            Vec3::new(0.0, 0.0, 0.0),
            asset_server.load("sprites/Ice.png"),
        ))
        .id();
    let p1_handle = std::thread::spawn(move || {
        let _ = p1.run();
    });
    orchestrator.add_planet_handle(id, p1_handle);

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
        ],
    ));

    let width = percent(25.0);
    let height = percent(50.0);

    commands.spawn((
        DespawnOnExit(GameState::Creative),
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: percent(5.0),
            width: percent(70.0),
            height: percent(10.0),
            left: percent(30),
            ..default()
        },
        children![(
            Node {
                width,
                height,
                border: UiRect::all(px(2.0)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(Color::WHITE),
            children![(
                Text::new("Sunray (S)"),
                theme::title_font(&asset_server),
                theme::text_color(),
            )],
        ),(
            Node {
                width,
                height,
                border: UiRect::all(px(2.0)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(Color::WHITE),
            children![(
                Text::new("Aseroid (A)"),
                theme::title_font(&asset_server),
                theme::text_color(),
            )],
        )],
    ));

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

fn create_event_listen(mut commands: Commands, planet: Single<Entity, With<Planet>>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::KeyS) {
        commands.spawn((
            DespawnOnExit(GameState::Creative),
            GalaxyEvent::Sunray,
            EventTarget {
                planet: *planet,
                duration: Timer::from_seconds(3.0, TimerMode::Once),
            },
        ));
    }

    if keyboard_input.just_pressed(KeyCode::KeyA) {
        commands.spawn((
            DespawnOnExit(GameState::Creative),
            GalaxyEvent::Asteroid,
            EventTarget {
                planet: *planet,
                duration: Timer::from_seconds(3.0, TimerMode::Once),
            },
        ));
    }
}

fn check_entities_and_end_game(
    planet: Query<&Planet>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !planet.is_empty() {
        return;
    }
    // No player entity found → end game
    next_state.set(GameState::Settings);
    // Update UI text instead of printing
}
