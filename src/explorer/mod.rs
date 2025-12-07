use bevy::prelude::Component;
use bevy::prelude::Entity;

pub mod movement;

#[derive(Component)]
pub struct Explorer {
    _target_planet: Option<Entity>,
    _travel_speed: f32,
}

#[derive(Component)]
pub struct Roaming;

#[derive(Component)]
pub struct Landed {
    pub(crate) planet: Entity,
}

impl Explorer {
    pub fn new(_target_planet: Option<Entity>, _travel_speed: f32) -> Self {
        Self {
            _target_planet,
            _travel_speed,
        }
    }
}
