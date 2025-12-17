use crate::GameState;
use crate::theme;
use bevy::prelude::*;

#[derive(Component)]
pub struct Planet; /*{
name: String,
position: Vec2,
}*/

/*impl Planet {
    pub fn new(name: &str, position: Vec2) -> Self {
        Self {
            name: name.to_string(),
            position,
        }
    }

    pub fn name(&self) -> String {
        String::from(&self.name)
    }

    pub fn position(&self) -> &Vec2 {
        &self.position
    }
}*/

#[derive(Component)]
pub struct PlanetId(pub u32);
#[derive(Component)]
pub struct PlanetName(pub String);
#[derive(Component)]
pub struct PlanetUi(pub Entity);
#[derive(Component)]
pub struct PlanetCell {
    pub num_cell: usize,
    pub charged_cell: usize,
}
#[derive(Component)]
pub struct PlanetRocket(pub bool);

pub fn cell_string(cell: &PlanetCell) -> String {
    let mut cells = String::new();
    cells.push_str(&"󰁹 ".repeat(cell.charged_cell));
    cells.push_str(&"󰁺 ".repeat(cell.num_cell - cell.charged_cell));
    cells
}

pub fn planet(id: u32, name: &str, position: Vec3, image: Handle<Image>) -> impl Bundle {
    (
        DespawnOnExit(GameState::Playing),
        Sprite {
            image: image,
            custom_size: Some(Vec2::new(100.0, 100.0)),
            ..default()
        },
        PlanetName(name.to_string()),
        Transform::from_translation(position),
        PlanetId(id),
        Planet,
    )
}

pub fn planet_state(
    asset_server: &Res<AssetServer>,
    planet_name: &str,
    planet: Entity,
    cell: PlanetCell,
    rocket: impl Component,
) -> impl Bundle {
    let padding = 12.0;
    let width = 90.0;
    let height = 15.0;

    (
        DespawnOnExit(GameState::Playing),
        Node {
            flex_direction: FlexDirection::Column,
            //top: top,
            //right: right,
            //left: left,
            padding: UiRect::all(Val::Px(padding)),
            width: Val::Percent(width),
            height: Val::Percent(height),
            ..default()
        },
        //state,
        PlanetUi(planet),
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
                Text::new(cell_string(&cell)),
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
