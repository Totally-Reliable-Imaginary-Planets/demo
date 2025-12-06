use bevy::prelude::Component;
use bevy::prelude::Entity;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::Time;
use bevy::prelude::Transform;
use bevy::prelude::Vec2;

use crate::PlanetEntities;
use crate::planet::Planet;

#[derive(Component)]
pub struct Explorer {
    target_planet: Option<Entity>,
    travel_speed: f32,
}

impl Explorer {
    pub fn new(target_planet: Option<Entity>, travel_speed: f32) -> Self {
        Self {
            target_planet,
            travel_speed,
        }
    }
}

pub fn explorer_movement_system(
    time: Res<Time>,
    mut explorer_query: Query<(&mut Transform, &mut Explorer)>,
    planet_query: Query<&Planet>,
    planet_entities: Res<PlanetEntities>,
) {
    for (mut transform, mut explorer) in explorer_query.iter_mut() {
        if let Some(target_entity) = explorer.target_planet {
            if let Ok(target_planet) = planet_query.get(target_entity) {
                let current_pos = Vec2::new(transform.translation.x, transform.translation.y);
                let direction = target_planet.position() - current_pos;
                let distance = direction.length();

                if distance > 5.0 {
                    // Still traveling
                    let movement =
                        direction.normalize() * explorer.travel_speed * time.delta_secs();
                    transform.translation.x += movement.x;
                    transform.translation.y += movement.y;
                } else {
                    // Arrived, switch to other planet
                    let other_planet = planet_entities
                        .planets
                        .iter()
                        .find(|&&p| p != target_entity)
                        .copied();
                    explorer.target_planet = other_planet;
                    println!("Explorer arrived at {}!", target_planet.name());
                }
            }
        }
    }
}
