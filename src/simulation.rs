use bevy::prelude::*;

use crate::explorer::Explorer;
use crate::planet::Planet;
use crate::resources::EventSpawnTimer;
use crate::resources::PlanetEntities;

use super::GameState;

pub fn simulation_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Playing), setup)
        .add_systems(
            Update,
            (
                check_entities_and_end_game,
                crate::explorer::movement::explorer_movement_system_wasd,
                crate::explorer::movement::check_explorer_reach,
                crate::galaxy_event::event_spawner_system,
                crate::galaxy_event::event_handler_system,
                crate::galaxy_event::cleanup_events_system,
                crate::galaxy_event::event_visual_system,
            )
                .run_if(in_state(GameState::Playing)),
        );
}

#[derive(Component)]
pub struct LogText;

#[derive(Component)]
pub struct PlanetDialog;

// Marker components for buttons
#[derive(Component)]
struct YesButton;

#[derive(Component)]
struct NoButton;

fn setup(mut commands: Commands) {
    // Planets
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
        Transform::from_xyz(0.0, 0.0, 1.0),
        Explorer::new(Some(planet2), 150.0),
    ));


    // Spawn dialog UI
    commands.spawn(dialog());

    // Resources
    commands.insert_resource(EventSpawnTimer(Timer::from_seconds(
        5.0,
        TimerMode::Repeating,
    )));
    commands.insert_resource(PlanetEntities {
        planets: vec![planet1, planet2],
    });
}

fn check_entities_and_end_game(
    mut commands: Commands,
    planet: Query<&Planet>,
    mut next_state: ResMut<NextState<GameState>>,
    query: Query<Entity, With<Explorer>>,
    mut log_query: Query<&mut Text, With<LogText>>,
) {
    if planet.is_empty() {
        for entity in &query {
            commands.entity(entity).despawn();
        }
        // No player entity found → end game
        next_state.set(GameState::Settings);
        // Update UI text instead of printing
        if let Ok(mut text) = log_query.single_mut() {
            text.0 = format!("{}\n{}", "GameOver", text.0);
        }
    }
}

fn dialog() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Percent(35.0),
            left: Val::Percent(35.0),
            width: Val::Percent(40.0),
            height: Val::Percent(40.0),
            ..default()
        },
        BackgroundColor(Color::BLACK.with_alpha(0.7)),
        PlanetDialog,
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),

                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.7)),
            children![
                (
                    Text::new("You have reached a planet do you want to land on it?"),
                    TextFont::default().with_font_size(16.0),
                    TextColor(Color::WHITE),
                ),
                (
                    Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    children![
                        (
                            Button,
                            Node {
                                width: px(150),
                                height: px(65),
                                border: UiRect::all(px(5)),
                                // horizontally center child text
                                justify_content: JustifyContent::Center,
                                // vertically center child text
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BorderColor::all(Color::WHITE),
                            YesButton,
                            children![(
                                Text::new("Yes"),
                                TextFont::default().with_font_size(16.0),
                                TextColor(Color::WHITE),
                            )]
                        ),
                        (
                            Button,
                            Node {
                                width: px(150),
                                height: px(65),
                                border: UiRect::all(px(5)),
                                // horizontally center child text
                                justify_content: JustifyContent::Center,
                                // vertically center child text
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BorderColor::all(Color::WHITE),
                            NoButton,
                            children![(
                                Text::new("No"),
                                TextFont::default().with_font_size(16.0),
                                TextColor(Color::WHITE),
                            )],
                        )
                    ],
                )
            ],
        )],
    )
}
