use bevy::prelude::Entity;
use bevy::prelude::Timer;
use bevy::prelude::Resource;

#[derive(Resource)]
pub struct EventSpawnTimer(pub(crate) Timer);

#[derive(Resource)]
pub struct PlanetEntities {
    pub(crate) planets: Vec<Entity>,
}
