use bevy::prelude::Entity;
use bevy::prelude::Resource;
use bevy::prelude::Timer;

#[derive(Resource, Default)]
pub struct EventSpawnTimer(pub(crate) Timer);

#[derive(Resource)]
pub struct PlanetEntities {
    pub(crate) planets: Vec<Entity>,
}
