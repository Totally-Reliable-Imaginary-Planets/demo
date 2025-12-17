use crate::GameState;
use crate::planet::*;
use bevy::prelude::*;

pub fn simulation_better_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Playing), setup);
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

    commands.spawn(planet_state(
        &asset_server,
        "Alpha",
        planet1,
        PlanetCell {
            num_cell: 5,
            charged_cell: 0,
        },
        PlanetRocket(false),
    ));
}
