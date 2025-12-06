use bevy::prelude::Entity;
use bevy::prelude::Resource;
use bevy::prelude::Timer;

#[derive(Resource)]
pub struct EventSpawnTimer(pub(crate) Timer);

#[derive(Resource)]
pub struct PlanetEntities {
    pub(crate) planets: Vec<Entity>,
}
