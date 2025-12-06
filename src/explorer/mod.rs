use bevy::prelude::Component;
use bevy::prelude::Entity;

pub mod movement;

#[derive(Component)]
pub struct Explorer {
    _target_planet: Option<Entity>,
    _travel_speed: f32,
}

impl Explorer {
    pub fn new(_target_planet: Option<Entity>, _travel_speed: f32) -> Self {
        Self {
            _target_planet,
            _travel_speed,
        }
    }
}
